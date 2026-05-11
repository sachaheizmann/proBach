import Sumcheck.Src.Transcript
import Sumcheck.Src.Hypercube
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic
import Std

-- ─── Setup ───────────────────────────────────────────────────────────────────

axiom koalabear_prime : Nat.Prime 2130706433
instance : Fact (Nat.Prime 2130706433) := ⟨koalabear_prime⟩
set_option maxHeartbeats 10000000
open Std

def boolDomainKB : List (ZMod 2130706433) := [0, 1]

-- ─── Helpers ─────────────────────────────────────────────────────────────────

def parseNatListKB (s : String) : List Nat :=
  s.splitOn " " |>.filterMap String.toNat?

-- ─── Pure Computation ────────────────────────────────────────────────────────

@[export lean_compute_transcript_koalabear]
def computeTranscriptKB
    (n          : Nat)
    (terms      : List (Nat × List Nat))
    (challenges : List Nat)
    : (List Nat) × (List (Nat × Nat)) :=
  let poly : CPoly.Unlawful n (ZMod 2130706433) :=
    terms.foldl (fun acc (c, exps) =>
      let coeff     := (c : ZMod 2130706433)
      let exponents := (List.range n).map (fun i => exps.getD i 0) |>.toArray
      let mon       : CPoly.CMvMonomial n := ⟨exponents, by simp [exponents]⟩
      acc.insert mon coeff
    ) 0
  let lawfulPoly   := CPoly.Lawful.fromUnlawful poly
  let chal         : Fin n → ZMod 2130706433 := fun i => (challenges[i.val]! : ZMod 2130706433)
  let initialClaim := honestClaim boolDomainKB lawfulPoly
  let transcript   :=
    generateHonestTranscript (𝔽 := ZMod 2130706433) boolDomainKB lawfulPoly initialClaim chal
  let claims := (List.finRange (n + 1)).map
    fun i => (transcript.claims initialClaim i).val
  let roundPolys := (List.finRange n).map fun i =>
    let p  := transcript.roundPolys i
    let v0 := CPoly.CMvPolynomial.eval (fun _ => (0 : ZMod 2130706433)) p
    let v1 := CPoly.CMvPolynomial.eval (fun _ => (1 : ZMod 2130706433)) p
    (v0.val, v1.val)
  (claims, roundPolys)

-- ─── IO Layer ────────────────────────────────────────────────────────────────

def runTestCaseKB (tc : String) : IO Unit := do
  let lines    := (tc.splitOn "\n" |>.filter (· ≠ "")).toArray
  if lines.size == 0 then return
  let n        := lines[0]!.toNat!
  let numTerms := lines[1]!.toNat!
  let termLines := lines.toList.drop 2 |>.take numTerms
  let terms := termLines.map fun line =>
    let nums := parseNatListKB line
    (nums[0]!, nums.drop 1)
  let challengeVals := parseNatListKB lines[2 + numTerms]!
  let (claims, roundPolys) := computeTranscriptKB n terms challengeVals
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
        runTestCaseKB (buffer.toList |> String.intercalate "\n")
    else
      let trimmed := line.trimAscii.toString
      if trimmed == "---" then
        if buffer.size > 0 then do
          runTestCaseKB (buffer.toList |> String.intercalate "\n")
          buffer := #[]
      else
        buffer := buffer.push trimmed
