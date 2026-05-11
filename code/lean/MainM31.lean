import Sumcheck.Src.Transcript
import Sumcheck.Src.Hypercube
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic
import Std

-- ─── Setup ───────────────────────────────────────────────────────────────────

axiom mersenne31_prime : Nat.Prime 2147483647
instance : Fact (Nat.Prime 2147483647) := ⟨mersenne31_prime⟩
set_option maxHeartbeats 10000000
open Std

def boolDomainM31 : List (ZMod 2147483647) := [0, 1]

-- ─── Helpers ─────────────────────────────────────────────────────────────────

def parseNatListM31 (s : String) : List Nat :=
  s.splitOn " " |>.filterMap String.toNat?

-- ─── Pure Computation ────────────────────────────────────────────────────────

@[export lean_compute_transcript_m31]
def computeTranscriptM31
    (n          : Nat)
    (terms      : List (Nat × List Nat))
    (challenges : List Nat)
    : (List Nat) × (List (Nat × Nat)) :=
  let poly : CPoly.Unlawful n (ZMod 2147483647) :=
    terms.foldl (fun acc (c, exps) =>
      let coeff     := (c : ZMod 2147483647)
      let exponents := (List.range n).map (fun i => exps.getD i 0) |>.toArray
      let mon       : CPoly.CMvMonomial n := ⟨exponents, by simp [exponents]⟩
      acc.insert mon coeff
    ) 0
  let lawfulPoly   := CPoly.Lawful.fromUnlawful poly
  let chal         : Fin n → ZMod 2147483647 := fun i => (challenges[i.val]! : ZMod 2147483647)
  let initialClaim := honestClaim boolDomainM31 lawfulPoly
  let transcript   :=
    generateHonestTranscript (𝔽 := ZMod 2147483647) boolDomainM31 lawfulPoly initialClaim chal
  let claims := (List.finRange (n + 1)).map
    fun i => (transcript.claims initialClaim i).val
  let roundPolys := (List.finRange n).map fun i =>
    let p  := transcript.roundPolys i
    let v0 := CPoly.CMvPolynomial.eval (fun _ => (0 : ZMod 2147483647)) p
    let v1 := CPoly.CMvPolynomial.eval (fun _ => (1 : ZMod 2147483647)) p
    (v0.val, v1.val)
  (claims, roundPolys)

-- ─── IO Layer ────────────────────────────────────────────────────────────────

def runTestCaseM31 (tc : String) : IO Unit := do
  let lines    := (tc.splitOn "\n" |>.filter (· ≠ "")).toArray
  if lines.size == 0 then return
  let n        := lines[0]!.toNat!
  let numTerms := lines[1]!.toNat!
  let termLines := lines.toList.drop 2 |>.take numTerms
  let terms := termLines.map fun line =>
    let nums := parseNatListM31 line
    (nums[0]!, nums.drop 1)
  let challengeVals := parseNatListM31 lines[2 + numTerms]!
  let (claims, roundPolys) := computeTranscriptM31 n terms challengeVals
  let claimsStr    := ", ".intercalate (claims.map toString)
  let challengeStr := ", ".intercalate (challengeVals.map toString)
  let mut out := "=== SUMCHECK TRANSCRIPT ===\n"
  out := out ++ s!"claims: [{claimsStr}]\n"
  out := out ++ s!"challenges: [{challengeStr}]\n"
  for (i, (v0, v1)) in (List.range roundPolys.length).zip roundPolys do
    out := out ++ s!"round_poly_{i}: [{v0}, {v1}]\n"
  out := out ++ "=== END ===\n"
  IO.print out
  (← IO.getStdout).flush

-- ─── Entry Point ─────────────────────────────────────────────────────────────

def main : IO Unit := do
  let stdin       := (← IO.getStdin)
  let mut buffer  : Array String := #[]
  let mut running := true
  while running do
    let line := (← stdin.getLine)
    if line == "" then do
      running := false
      if buffer.size > 0 then
        runTestCaseM31 (buffer.toList |> String.intercalate "\n")
    else
      let trimmed := line.trimAscii.toString
      if trimmed == "---" then
        if buffer.size > 0 then do
          runTestCaseM31 (buffer.toList |> String.intercalate "\n")
          buffer := #[]
      else
        buffer := buffer.push trimmed
