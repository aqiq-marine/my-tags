// Query DSL example: type filters, property predicates, and set operations.

def Video: Subject
def ChemObject: Video
def System
def Solvent
def Solution {
  solvent: Solvent
}: System
def Neat: System
def ReactionCondition {
  temperature: Float,
  concentration: Float,
  system: System
}
def Reaction: Tag {
  yield: Float
  condition: ReactionCondition
}
def Asymmetric: Reaction {
  ee: Float
}

let neat: Neat
let methanol: Solvent
let ethanol: Solvent
let heck: Asymmetric
let reaction_a: ChemObject
let reaction_b: ChemObject
let reaction_c: ChemObject

stmt reaction_a has heck {
  yield: 90
  ee: 80
  condition: ReactionCondition { temperature: 25.0, concentration: 0.125, system: Solution { solvent: methanol } }
}

stmt reaction_b has heck {
  yield: 70
  ee: 95
  condition: ReactionCondition { temperature: 30, concentration: 0.25, system: Solution { solvent: ethanol } }
}

stmt reaction_c has heck {
  ee: 90
  condition: ReactionCondition { temperature: 25, concentration: 0.125, system: Solution { solvent: methanol } }
}

#show query Reaction {
  yield: (>= 80)
  condition: ReactionCondition {
    system: Solution { solvent: methanol }
  }
}

#show query Reaction {
  yield: none
}
