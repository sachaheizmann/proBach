import Sumcheck.Src.Transcript
import Sumcheck.Src.Hypercube
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic

-- ─── Setup ───────────────────────────────────────────────────────────────────

instance : Fact (Nat.Prime 19) := ⟨by decide⟩

axiom mersenne31_prime : Nat.Prime 2147483647
instance : Fact (Nat.Prime 2147483647) := ⟨mersenne31_prime⟩

axiom babybear_prime : Nat.Prime 2013265921
instance : Fact (Nat.Prime 2013265921) := ⟨babybear_prime⟩

axiom koalabear_prime : Nat.Prime 2130706433
instance : Fact (Nat.Prime 2130706433) := ⟨koalabear_prime⟩

axiom goldilocks_prime : Nat.Prime 18446744069414584321
instance : Fact (Nat.Prime 18446744069414584321) := ⟨goldilocks_prime⟩

set_option maxHeartbeats 10000000

-- ─── Pure Computation ────────────────────────────────────────────────────────

private def runProtocol (p : Nat) [Fact (Nat.Prime p)] (n : Nat)
  (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=

  let boolDom : List (ZMod p) := [0, 1]

  let poly : CPoly.Unlawful n (ZMod p) :=
    terms.foldl (fun acc (c, exps) =>
      let coeff     := (c : ZMod p)
      let exponents := (List.range n).map (fun i => exps.getD i 0) |>.toArray
      let mon       : CPoly.CMvMonomial n := ⟨exponents, by simp [exponents]⟩
      acc.insert mon coeff
    ) 0

  let lawfulPoly   := CPoly.Lawful.fromUnlawful poly

  let chal         : Fin n → ZMod p := fun i => (challenges[i.val]! : ZMod p)

  let initialClaim := honestClaim boolDom lawfulPoly

  let transcript   :=
    generateHonestTranscript (𝔽 := ZMod p) boolDom lawfulPoly initialClaim chal

  let s0List := (List.finRange n).map fun i =>
      let poly := transcript.roundPolys i
      UInt64.ofNat (CPoly.CMvPolynomial.eval (fun _ => (0 : ZMod p)) poly).val

  let finalValue := UInt64.ofNat (transcript.claims initialClaim ⟨n, Nat.lt_succ_self n⟩).val

  (s0List, finalValue)


-- ─── FFI Exports ─────────────────────────────────────────────────────────────

@[export lean_compute_transcript_z19]
def computeTranscriptZ19 (n : Nat) (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=
  runProtocol 19 n terms challenges

@[export lean_compute_transcript_m31]
def computeTranscriptM31 (n : Nat) (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=
  runProtocol 2147483647 n terms challenges

@[export lean_compute_transcript_babybear]
def computeTranscriptBB (n : Nat) (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=
  runProtocol 2013265921 n terms challenges

@[export lean_compute_transcript_koalabear]
def computeTranscriptKB (n : Nat) (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=
  runProtocol 2130706433 n terms challenges

@[export lean_compute_transcript_goldilocks]
def computeTranscriptGL (n : Nat) (terms : List (Nat × List Nat)) (challenges : List Nat)
    : List UInt64 × UInt64 :=
  runProtocol 18446744069414584321 n terms challenges
