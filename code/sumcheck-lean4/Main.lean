import Sumcheck.Src.Transcript
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Std

instance : Fact (Nat.Prime 19) := ⟨by decide⟩

set_option maxHeartbeats 10000000

open Std

def boolDomain : List (ZMod 19) := [0,1]


def parseNatList (s : String) : List Nat :=
  s.splitOn " " |>.filterMap String.toNat?

def main : IO Unit := do

  let stdin ← IO.getStdin
  let content ← stdin.readToEnd
  let lines := (content.splitOn "\n" |>.filter (· ≠ "")).toArray
  -- parse n
  let n := (lines[0]!).toNat!

  -- parse number of terms
  let numTerms := (lines[1]!).toNat!

  -- parse polynomial
  let termLines := (lines.toList.drop 2 |>.take numTerms)

  let mut poly : CPoly.Unlawful n (ZMod 19) := 0

  for line in termLines do
    let nums := parseNatList line
    let coeff := (nums[0]! : ZMod 19)

    let raw := nums.drop 1

    let exponents :=
      (List.range n).map (fun i => raw.getD i 0) |>.toArray

    let mon : CPoly.CMvMonomial n :=
      ⟨exponents, by
        simp [exponents]
      ⟩

    poly := poly.insert mon coeff

  let examplePoly := CPoly.Lawful.fromUnlawful poly

  -- sum poly over boolDomain^n
  let initialClaim : ZMod 19 :=
    sum_over_domain_recursive
      boolDomain
      (add := (· + ·))
      (zero := 0)
      (m := n)
      (F := fun b =>
        CPoly.CMvPolynomial.eval₂ (RingHom.id _)
          (fun i => b i) examplePoly)

  -- parse challenges
  let challengeLine := lines[2 + numTerms]!
  let challengeVals := parseNatList challengeLine

  let challenges : Fin n → ZMod 19 :=
    fun i => (challengeVals[i.val]! : ZMod 19)

  let transcript :=
    generate_honest_transcript
      (𝔽 := ZMod 19)
      boolDomain
      examplePoly
      (honest_claim boolDomain examplePoly)
      challenges

  IO.println "=== SUMCHECK TRANSCRIPT ==="

  -- claims
  IO.print "claims: ["
  for i in List.finRange (n + 1) do
    let val := (transcript.claims i).val
    if i.val > 0 then IO.print ", "
    IO.print s!"{val}"
  IO.println "]"

  -- challenges
  IO.print "challenges: ["
  for i in List.finRange n do
    let val := (transcript.challenges i).val
    if i.val > 0 then IO.print ", "
    IO.print s!"{val}"
  IO.println "]"

  -- round polys
  for i in List.finRange n do
    let p := transcript.round_polys i

    let v0 := CPoly.CMvPolynomial.eval₂ (RingHom.id _) (fun _ => (0 : ZMod 19)) p
    let v1 := CPoly.CMvPolynomial.eval₂ (RingHom.id _) (fun _ => (1 : ZMod 19)) p

    IO.println s!"round_poly_{i.val}: [{v0.val}, {v1.val}]"

  IO.println "=== END ==="
