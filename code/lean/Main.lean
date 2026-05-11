import Sumcheck.Src.Transcript
import Sumcheck.Src.Hypercube
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic
import Std

-- ─── Setup ───────────────────────────────────────────────────────────────────

instance : Fact (Nat.Prime 19) := ⟨by decide⟩
set_option maxHeartbeats 10000000
open Std

def boolDomain : List (ZMod 19) := [0, 1]

-- ─── Helpers ─────────────────────────────────────────────────────────────────

def parseNatList (s : String) : List Nat :=
  s.splitOn " " |>.filterMap String.toNat?

-- ─── Pure Computation ────────────────────────────────────────────────────────
@[export lean_compute_transcript_z19]
def computeTranscript
    (n          : Nat)
    (terms      : List (Nat × List Nat))
    (challenges : List Nat)
    : (List Nat) × (List (Nat × Nat)) :=
  -- build polynomial using fold instead of mut + for loop
  let poly : CPoly.Unlawful n (ZMod 19) :=
    terms.foldl (fun acc (c, exps) =>
      let coeff     := (c : ZMod 19)
      let exponents := (List.range n).map (fun i => exps.getD i 0) |>.toArray
      let mon       : CPoly.CMvMonomial n := ⟨exponents, by simp [exponents]⟩
      acc.insert mon coeff
    ) 0
  -- run sumcheck protocol
  let lawfulPoly   := CPoly.Lawful.fromUnlawful poly
  let chal         : Fin n → ZMod 19 := fun i => (challenges[i.val]! : ZMod 19)
  let initialClaim := honestClaim boolDomain lawfulPoly
  let transcript   :=
    generateHonestTranscript (𝔽 := ZMod 19) boolDomain lawfulPoly initialClaim chal
  -- extract claims
  let claims := (List.finRange (n + 1)).map
    fun i => (transcript.claims initialClaim i).val
  -- extract round polynomials as (p0, p1) pairs
  let roundPolys := (List.finRange n).map fun i =>
    let p  := transcript.roundPolys i
    let v0 := CPoly.CMvPolynomial.eval (fun _ => (0 : ZMod 19)) p
    let v1 := CPoly.CMvPolynomial.eval (fun _ => (1 : ZMod 19)) p
    (v0.val, v1.val)
  (claims, roundPolys)

-- ─── Export for FFI ──────────────────────────────────────────────────────────

@[export lean_sumcheck_z19]
def leanSumcheckZ19
    (n          : UInt32)
    (terms      : @&Array (UInt64 × Array UInt64))
    (challenges : @&Array UInt64)
    : Array UInt64 :=
  -- convert UInt32/UInt64 inputs to Nat
  let nNat       := n.toNat
  let termsNat   := terms.toList.map fun (c, exps) =>
    (c.toNat, exps.toList.map UInt64.toNat)
  let chalNat    := challenges.toList.map UInt64.toNat
  -- run pure computation
  let (claims, roundPolys) := computeTranscript nNat termsNat chalNat
  -- encode output as flat UInt64 array:
  -- [claim_0, claim_1, ..., claim_n, p0_0, p1_0, p0_1, p1_1, ...]
  let claimsArr   := claims.map (fun x => UInt64.ofNat x)
  let roundArr    := roundPolys.flatMap (fun (p0, p1) => [UInt64.ofNat p0, UInt64.ofNat p1])
  (claimsArr ++ roundArr).toArray

-- ─── IO Layer ────────────────────────────────────────────────────────────────

def runTestCase (tc : String) : IO Unit := do
  let lines    := (tc.splitOn "\n" |>.filter (· ≠ "")).toArray
  if lines.size == 0 then return
  let n        := lines[0]!.toNat!
  let numTerms := lines[1]!.toNat!
  -- parse terms
  let termLines := lines.toList.drop 2 |>.take numTerms
  let terms := termLines.map fun line =>
    let nums := parseNatList line
    (nums[0]!, nums.drop 1)
  -- parse challenges
  let challengeVals := parseNatList lines[2 + numTerms]!
  -- compute transcript
  let (claims, roundPolys) := computeTranscript n terms challengeVals
  -- format output
  let claimsStr    := ", ".intercalate (claims.map toString)
  let challengeStr := ", ".intercalate (challengeVals.map toString)
  let mut out := "=== SUMCHECK TRANSCRIPT ===\n"
  out := out ++ s!"claims: [{claimsStr}]\n"
  out := out ++ s!"challenges: [{challengeStr}]\n"
  for (i, (v0, v1)) in (List.range roundPolys.length).zip roundPolys do
    out := out ++ s!"round_poly_{i}: [{v0}, {v1}]\n"
  out := out ++ "=== END ===\n"
  -- print and flush atomically
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
        runTestCase (buffer.toList |> String.intercalate "\n")
    else
      let trimmed := line.trimAscii.toString
      if trimmed == "---" then
        if buffer.size > 0 then do
          runTestCase (buffer.toList |> String.intercalate "\n")
          buffer := #[]
      else
        buffer := buffer.push trimmed
