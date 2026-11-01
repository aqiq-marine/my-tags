// Query DSL example: type filters, property predicates, and set operations.

def Video
def Solvent
def SolventInfo { solvent: Solvent }
def Reaction: Tag {
  yield: Number
}
def Asymmetric: Reaction {
  ee: Number
}
def Hoge {
  huga: SolventInfo
}

let methanol: Solvent
let ethanol: Solvent
let heck: Asymmetric + Hoge
let video_a: Video
let video_b: Video
let video_c: Video

stmt video_a has heck {
  yield: 90
  ee: 80
  huga: SolventInfo { solvent: methanol }
}

stmt video_b has heck {
  yield: 70
  ee: 95
  huga: SolventInfo { solvent: ethanol }
}

stmt video_c has heck {
  ee: 90
  huga: SolventInfo { solvent: methanol }
}

#show query Reaction {
  yield: (>= 80)
} & Hoge {
  huga: SolventInfo {
    solvent: methanol
  }
}

#show query Reaction {
  yield: none
}
