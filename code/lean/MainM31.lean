import Sumcheck.Src.Transcript
import Sumcheck.Src.Hypercube
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic
import Std

axiom mersenne31_prime : Nat.Prime 2147483647
instance : Fact (Nat.Prime 2147483647) := ⟨mersenne31_prime⟩

set_option maxHeartbeats 10000000
open Std

def boolDomainM31 : List (ZMod 2147483647) := [0,1]

def parseNatListM31 (s : String) : List Nat :=
  s.splitOn " " |>.filterMap String.toNat?

def main : IO Unit := do
  let stdin ← IO.getStdin
  let content ← stdin.readToEnd
  let testCases := content.splitOn "---"
    |>.map String.trim
    |>.filter (· ≠ "")
  let mut idx := 0
  for tc in testCases do
    idx := idx + 1
    IO.println s!"=== TEST {idx} ==="
    let lines := (tc.splitOn "\n" |>.filter (· ≠ "")).toArray
    let n := (lines[0]!).toNat!
    let numTerms := (lines[1]!).toNat!
    let termLines := (lines.toList.drop 2 |>.take numTerms)
    let mut poly : CPoly.Unlawful n (ZMod 2147483647) := 0
    for line in termLines do
      let nums := parseNatListM31 line
      let coeff := (nums[0]! : ZMod 2147483647)
      let raw := nums.drop 1
      let exponents :=
        (List.range n).map (fun i => raw.getD i 0) |>.toArray
      let mon : CPoly.CMvMonomial n :=
        ⟨exponents, by simp [exponents]⟩
      poly := poly.insert mon coeff
    let examplePoly := CPoly.Lawful.fromUnlawful poly
    let challengeLine := lines[2 + numTerms]!
    let challengeVals := parseNatListM31 challengeLine
    let challenges : Fin n → ZMod 2147483647 :=
      fun i => (challengeVals[i.val]! : ZMod 2147483647)
    let transcript :=
      generate_honest_transcript
        (𝔽 := ZMod 2147483647)
        boolDomainM31
        examplePoly
        (honest_claim boolDomainM31 examplePoly)
        challenges
    IO.println "=== SUMCHECK TRANSCRIPT ==="
    IO.print "claims: ["
    for i in List.finRange (n + 1) do
      let val := (transcript.claims i).val
      if i.val > 0 then IO.print ", "
      IO.print s!"{val}"
    IO.println "]"
    IO.print "challenges: ["
    for i in List.finRange n do
      let val := (transcript.challenges i).val
      if i.val > 0 then IO.print ", "
      IO.print s!"{val}"
    IO.println "]"
    for i in List.finRange n do
      let p := transcript.round_polys i
      let v0 := CPoly.CMvPolynomial.eval₂ (RingHom.id _) (fun _ => (0 : ZMod 2147483647)) p
      let v1 := CPoly.CMvPolynomial.eval₂ (RingHom.id _) (fun _ => (1 : ZMod 2147483647)) p
      IO.println s!"round_poly_{i.val}: [{v0.val}, {v1.val}]"
    IO.println "=== END ==="
