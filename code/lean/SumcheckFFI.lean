import SumcheckProtocol.Src.MultilinearProver
import CompPoly.Multivariate.CMvPolynomial
import Mathlib.Data.ZMod.Basic
import Mathlib.FieldTheory.Finite.Basic

-- ─── Setup ───────────────────────────────────────────────────────────────────
instance : Fact (Nat.Prime 19) := ⟨by decide⟩
axiom mersenne31_prime  : Nat.Prime 2147483647
axiom babybear_prime    : Nat.Prime 2013265921
axiom koalabear_prime   : Nat.Prime 2130706433
axiom goldilocks_prime  : Nat.Prime 18446744069414584321
instance : Fact (Nat.Prime 2147483647)           := ⟨mersenne31_prime⟩
instance : Fact (Nat.Prime 2013265921)           := ⟨babybear_prime⟩
instance : Fact (Nat.Prime 2130706433)           := ⟨koalabear_prime⟩
instance : Fact (Nat.Prime 18446744069414584321) := ⟨goldilocks_prime⟩

set_option maxHeartbeats 10000000

open SumcheckProtocol.MultilinearProver

-- ─── Fold helper ─────────────────────────────────────────────────────────────
-- Folds the eval table n times with the given challenges.
-- After n folds the single remaining entry is p(r0,...,r_{n-1}).
private def foldAll (p : Nat) [Fact (Nat.Prime p)]
    (challenges : List UInt64) :
    (m : Nat) → EvalTable m (ZMod p) → Nat → ZMod p
  | 0,     t, _  => t.get ⟨0, by norm_num⟩
  | m + 1, t, ci =>
      let r := ((challenges.getD ci 0).toNat : ZMod p)
      foldAll p challenges m (fold_msb_succ r t) (ci + 1)
-- ─── Pure Computation ────────────────────────────────────────────────────────
private def runProtocol (p : Nat) [Fact (Nat.Prime p)] (n : Nat)
    (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=

  -- Build EvalTable n (ZMod p) from flat list of 2^n field elements
  let arr := evalTable.toArray
  let table : EvalTable n (ZMod p) :=
    Vector.ofFn (fun i : Fin (2^n) =>
      ((arr.getD i.val 0).toNat : ZMod p))

  -- Build challenge function Fin n → ZMod p
  let chal : Fin n → ZMod p :=
    fun i => ((challenges.getD i.val 0).toNat : ZMod p)

  -- Run the eval-form prover: List (s0, s1) one pair per round
  let rounds : List (ZMod p × ZMod p) :=
    multilinearProverEvalForm chal table

  -- Extract s0 per round for output
  let s0List : List UInt64 :=
    rounds.map (fun (s0, _) => UInt64.ofNat s0.val)

  -- Final value: fold the table n times with all challenges
  let finalValue : UInt64 :=
    UInt64.ofNat (foldAll p challenges n table 0).val

  (s0List, finalValue)

-- ─── FFI Exports ─────────────────────────────────────────────────────────────
@[export lean_compute_transcript_z19]
def computeTranscriptZ19 (n : Nat) (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=
  runProtocol 19 n evalTable challenges

@[export lean_compute_transcript_m31]
def computeTranscriptM31 (n : Nat) (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=
  runProtocol 2147483647 n evalTable challenges

@[export lean_compute_transcript_babybear]
def computeTranscriptBB (n : Nat) (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=
  runProtocol 2013265921 n evalTable challenges

@[export lean_compute_transcript_koalabear]
def computeTranscriptKB (n : Nat) (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=
  runProtocol 2130706433 n evalTable challenges

@[export lean_compute_transcript_goldilocks]
def computeTranscriptGL (n : Nat) (evalTable : List UInt64) (challenges : List UInt64)
    : List UInt64 × UInt64 :=
  runProtocol 18446744069414584321 n evalTable challenges
