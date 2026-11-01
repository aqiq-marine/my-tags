import "chemistry.mt" as chem

def Experiment: Subject
let experiment: Experiment

stmt experiment has chem::reaction
#show query chem::Reaction {
  yield: (>= 80)
}
