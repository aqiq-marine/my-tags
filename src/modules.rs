use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::{
        Item, Program, PropertyAssignment, PropertyCondition, PropertyPredicate, QueryExpr,
        QueryValue, TypeExpression, ValueExpression,
    },
    parse,
};

/// Loads an entry file and all of its transitive imports into one program.
///
/// Every non-builtin symbol is rewritten to a canonical module-qualified name
/// before semantic analysis. Import aliases are only visible in the importing
/// module and are not re-exported.
pub fn load_project(entry: impl AsRef<Path>) -> Result<Program, ModuleLoadError> {
    let entry = entry.as_ref().canonicalize().map_err(|source| ModuleLoadError::Io {
        path: entry.as_ref().to_path_buf(),
        source,
    })?;
    let root = entry.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut loader = Loader {
        root,
        modules: Vec::new(),
        states: HashMap::new(),
        stack: Vec::new(),
    };
    let root_id = loader.visit(entry)?;
    let mut combined = Program { items: Vec::new() };

    for (module_id, module) in loader.modules.iter().enumerate() {
        let module = module.as_ref().expect("visited module is populated");
        for item in &module.program.items {
            if matches!(item, Item::Import(_)) {
                continue;
            }
            // Query directives are executable entry-point instructions, not
            // declarations to inherit from imported libraries.
            if module_id != root_id && matches!(item, Item::ShowQuery(_)) {
                continue;
            }
            let mut item = item.clone();
            rewrite_item(
                &mut item,
                module_id,
                &module.aliases,
                &module.key,
            )?;
            combined.items.push(item);
        }
    }
    Ok(combined)
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleLoadError {
    #[error("could not read module {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    #[error("could not parse module {path}: {source}")]
    Parse { path: PathBuf, #[source] source: crate::diagnostic::FrontendError },
    #[error("cyclic import detected: {chain}")]
    Cycle { chain: String },
    #[error("module {module} imports {path}, but that file is already imported in this module")]
    DuplicateImport { module: PathBuf, path: PathBuf },
    #[error("module {module} declares import alias {alias} more than once")]
    DuplicateAlias { module: PathBuf, alias: String },
    #[error("module {module} refers to unknown import alias {alias} in {name}")]
    UnknownAlias { module: PathBuf, alias: String, name: String },
    #[error("invalid qualified name {0}; expected alias::name")]
    InvalidQualifiedName(String),
}

struct LoadedModule {
    path: PathBuf,
    key: String,
    program: Program,
    aliases: HashMap<String, String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState { Visiting, Visited }

struct Loader {
    root: PathBuf,
    modules: Vec<Option<LoadedModule>>,
    states: HashMap<PathBuf, VisitState>,
    stack: Vec<PathBuf>,
}

impl Loader {
    fn visit(&mut self, path: PathBuf) -> Result<usize, ModuleLoadError> {
        let path = path.canonicalize().map_err(|source| ModuleLoadError::Io {
            path: path.clone(),
            source,
        })?;
        if let Some(state) = self.states.get(&path) {
            if *state == VisitState::Visited {
                return Ok(self.modules.iter().position(|module| {
                    module.as_ref().is_some_and(|loaded| loaded.path == path)
                }).expect("visited module has an entry"));
            }
            let mut chain: Vec<String> = self.stack.iter().map(|item| item.display().to_string()).collect();
            chain.push(path.display().to_string());
            return Err(ModuleLoadError::Cycle { chain: chain.join(" -> ") });
        }

        let source = fs::read_to_string(&path).map_err(|source| ModuleLoadError::Io {
            path: path.clone(),
            source,
        })?;
        let source_name = path.display().to_string();
        let program = parse(&source_name, &source)
            .map_err(|source| ModuleLoadError::Parse { path: path.clone(), source })?;
        let key = path.strip_prefix(&self.root).unwrap_or(&path)
            .to_string_lossy().replace('\\', "/");
        let id = self.modules.len();
        self.modules.push(Some(LoadedModule {
            path: path.clone(),
            key,
            program: program.clone(),
            aliases: HashMap::new(),
        }));
        self.states.insert(path.clone(), VisitState::Visiting);
        self.stack.push(path.clone());

        let imports: Vec<_> = program.items.iter().filter_map(|item| {
            if let Item::Import(import) = item { Some(import.clone()) } else { None }
        }).collect();
        let mut aliases = HashMap::new();
        let mut imported_paths = HashSet::new();
        for import in imports {
            if aliases.contains_key(&import.alias) {
                return Err(ModuleLoadError::DuplicateAlias {
                    module: path.clone(),
                    alias: import.alias,
                });
            }
            let target = path.parent().unwrap_or(Path::new(".")).join(&import.path);
            let canonical_target = target.canonicalize().map_err(|source| ModuleLoadError::Io {
                path: target.clone(),
                source,
            })?;
            if !imported_paths.insert(canonical_target.clone()) {
                return Err(ModuleLoadError::DuplicateImport {
                    module: path.clone(),
                    path: canonical_target,
                });
            }
            let target_id = self.visit(canonical_target)?;
            let target_key = self.modules[target_id].as_ref()
                .expect("import target is loaded")
                .key.clone();
            aliases.insert(import.alias, target_key);
        }

        self.stack.pop();
        self.states.insert(path, VisitState::Visited);
        self.modules[id].as_mut().expect("module was inserted").aliases = aliases;
        Ok(id)
    }
}

fn canonical_name(module_key: &str, name: &str) -> String {
    format!("{module_key}::{name}")
}

fn resolve_name(
    raw: &str,
    module_key: &str,
    aliases: &HashMap<String, String>,
    is_type: bool,
) -> Result<String, ModuleLoadError> {
    if is_type && matches!(raw, "Subject" | "Tag" | "Float" | "Int" | "Nat") {
        return Ok(raw.to_owned());
    }
    if let Some((alias, name)) = raw.split_once("::") {
        if name.is_empty() || name.contains("::") {
            return Err(ModuleLoadError::InvalidQualifiedName(raw.to_owned()));
        }
        let target = aliases.get(alias).ok_or_else(|| ModuleLoadError::UnknownAlias {
            module: PathBuf::from(module_key),
            alias: alias.to_owned(),
            name: raw.to_owned(),
        })?;
        return Ok(canonical_name(target, name));
    }
    Ok(canonical_name(module_key, raw))
}

fn rewrite_type_expr(
    expression: &mut TypeExpression,
    module: &str,
    aliases: &HashMap<String, String>,
) -> Result<(), ModuleLoadError> {
    for term in &mut expression.terms {
        *term = resolve_name(term, module, aliases, true)?;
    }
    Ok(())
}

fn rewrite_item(
    item: &mut Item,
    module_id: usize,
    aliases: &HashMap<String, String>,
    module: &str,
) -> Result<(), ModuleLoadError> {
    let _ = module_id;
    match item {
        Item::Import(_) => {}
        Item::TypeDefinition(def) => {
            if !matches!(def.name.as_str(), "Subject" | "Tag" | "Float" | "Int" | "Nat") {\n                def.name = canonical_name(module, &def.name);\n            }
            if let Some(parent) = &mut def.parent {
                *parent = resolve_name(parent, module, aliases, true)?;
            }
            for property in &mut def.properties {
                rewrite_type_expr(&mut property.value_type, module, aliases)?;
            }
        }
        Item::InstanceDefinition(def) => {
            def.name = canonical_name(module, &def.name);
            rewrite_type_expr(&mut def.instance_type, module, aliases)?;
        }
        Item::Statement(statement) => {
            statement.subject = resolve_name(&statement.subject, module, aliases, false)?;
            statement.tag = resolve_name(&statement.tag, module, aliases, false)?;
            rewrite_assignments(&mut statement.properties, module, aliases)?;
        }
        Item::ShowQuery(query) => rewrite_query(&mut query.expression, module, aliases)?,
    }
    Ok(())
}

fn rewrite_assignments(
    assignments: &mut [PropertyAssignment],
    module: &str,
    aliases: &HashMap<String, String>,
) -> Result<(), ModuleLoadError> {
    for assignment in assignments {
        match &mut assignment.value {
            ValueExpression::Reference { name, .. } => {
                *name = resolve_name(name, module, aliases, false)?;
            }
            ValueExpression::Number { .. } => {}
            ValueExpression::AnonymousInstance { type_name, properties, .. } => {
                *type_name = resolve_name(type_name, module, aliases, true)?;
                rewrite_assignments(properties, module, aliases)?;
            }
        }
    }
    Ok(())
}

fn rewrite_query(
    expression: &mut QueryExpr,
    module: &str,
    aliases: &HashMap<String, String>,
) -> Result<(), ModuleLoadError> {
    match expression {
        QueryExpr::Type { name, conditions, .. } => {
            *name = resolve_name(name, module, aliases, true)?;
            rewrite_conditions(conditions, module, aliases)?;
        }
        QueryExpr::And(left, right)
        | QueryExpr::Or(left, right)
        | QueryExpr::Difference(left, right) => {
            rewrite_query(left, module, aliases)?;
            rewrite_query(right, module, aliases)?;
        }
        QueryExpr::Not(inner) => rewrite_query(inner, module, aliases)?,
    }
    Ok(())
}

fn rewrite_conditions(
    conditions: &mut [PropertyCondition],
    module: &str,
    aliases: &HashMap<String, String>,
) -> Result<(), ModuleLoadError> {
    for condition in conditions {
        rewrite_predicate(&mut condition.predicate, module, aliases)?;
    }
    Ok(())
}

fn rewrite_predicate(
    predicate: &mut PropertyPredicate,
    module: &str,
    aliases: &HashMap<String, String>,
) -> Result<(), ModuleLoadError> {
    match predicate {
        PropertyPredicate::None => {}
        PropertyPredicate::And(left, right) | PropertyPredicate::Or(left, right) => {
            rewrite_predicate(left, module, aliases)?;
            rewrite_predicate(right, module, aliases)?;
        }
        PropertyPredicate::Anonymous { type_name, conditions, .. } => {
            *type_name = resolve_name(type_name, module, aliases, true)?;
            rewrite_conditions(conditions, module, aliases)?;
        }
        PropertyPredicate::Compare { value, .. } => {
            if let QueryValue::Reference { name, .. } = value {
                *name = resolve_name(name, module, aliases, false)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::load_project;
    use crate::{resolve, ast::Item};

    fn write(path: &Path, source: &str) {
        fs::write(path, source).unwrap();
    }

    #[test]
    fn loads_transitive_modules_and_resolves_alias_qualified_symbols() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("chemistry.mt"), "def Atom\nlet atom: Tag + Atom\n");
        write(
            &dir.path().join("main.mt"),
            "import \"chemistry.mt\" as chem\ndef Video: Subject\nlet video: Video\nstmt video has chem::atom\n#show query chem::Atom {}",
        );

        let program = load_project(dir.path().join("main.mt")).unwrap();
        assert_eq!(program.items.iter().filter(|item| matches!(item, Item::Import(_))).count(), 0);
        let source = fs::read_to_string(dir.path().join("main.mt")).unwrap();
        let model = resolve(&program, "main.mt", &source).unwrap();
        assert!(model.types.iter().any(|ty| ty.name == "chemistry.mt::Atom"));
        assert!(model.instances.iter().any(|instance| instance.name == "chemistry.mt::atom"));
        assert_eq!(model.statements.len(), 1);
    }

    #[test]
    fn aliases_are_module_local_and_are_not_reexported() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("chemistry.mt"), "def Atom\n");
        write(
            &dir.path().join("catalog.mt"),
            "import \"chemistry.mt\" as chem\ndef Sample { atom: chem::Atom }\n",
        );
        write(
            &dir.path().join("main.mt"),
            "import \"catalog.mt\" as catalog\n#show query catalog::Sample {}",
        );
        let program = load_project(dir.path().join("main.mt")).unwrap();
        assert!(program.items.iter().any(|item| {
            matches!(item, Item::ShowQuery(_))
        }));

        write(
            &dir.path().join("main.mt"),
            "import \"catalog.mt\" as catalog\n#show query chem::Atom {}",
        );
        let error = load_project(dir.path().join("main.mt")).unwrap_err();
        assert!(error.to_string().contains("unknown import alias chem"));
    }

    #[test]
    fn rejects_duplicate_aliases() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("a.mt"), "def A\n");
        write(&dir.path().join("b.mt"), "def B\n");
        write(
            &dir.path().join("main.mt"),
            "import \"a.mt\" as lib\nimport \"b.mt\" as lib\n",
        );
        let error = load_project(dir.path().join("main.mt")).unwrap_err();
        assert!(error.to_string().contains("declares import alias lib more than once"));
    }

    #[test]
    fn rejects_duplicate_imports_of_the_same_file() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("a.mt"), "def A\n");
        write(
            &dir.path().join("main.mt"),
            "import \"a.mt\" as first\nimport \"./a.mt\" as second\n",
        );
        let error = load_project(dir.path().join("main.mt")).unwrap_err();
        assert!(error.to_string().contains("already imported"));
    }

    #[test]
    fn rejects_cyclic_imports_with_the_dependency_chain() {
        let dir = tempdir().unwrap();
        write(&dir.path().join("a.mt"), "import \"b.mt\" as b\n");
        write(&dir.path().join("b.mt"), "import \"a.mt\" as a\n");
        let error = load_project(dir.path().join("a.mt")).unwrap_err();
        assert!(error.to_string().contains("cyclic import detected"));
        assert!(error.to_string().contains("a.mt"));
        assert!(error.to_string().contains("b.mt"));
    }
}
