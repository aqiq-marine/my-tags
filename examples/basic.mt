// Subject, Tag, Float, Int, and Nat are built-in types.

def Video: Subject
def Appearance
def Color

def SolidColor {
  color: Color
}: Appearance

def Animal {
  appearance: Appearance
}

let white: Color
let black: Color
let cat: Tag + Animal
let video_a: Video

stmt video_a has cat {
  appearance: SolidColor {
    color: white
  }
}

stmt video_a has cat {
  appearance: SolidColor {
    color: black
  }
}
