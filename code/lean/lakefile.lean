import Lake
open Lake DSL

package "differential_testing" where
  version := v!"0.1.0"

require sumcheck from git "https://github.com/z-tech/z-lean" @ "main"

lean_lib MainShared where
  roots := #[`SumcheckFFI]
  defaultFacets := #[LeanLib.sharedFacet]

lean_exe sumcheck_z19 where
  root := `Main
  moreLinkArgs := #["-Wl,--export-dynamic"]
lean_exe sumcheck_m31 where
  root := `MainM31
lean_exe sumcheck_babybear where
  root := `MainBabyBear
lean_exe sumcheck_koalabear where
  root := `MainKoalaBear
lean_exe sumcheck_goldilocks where
  root := `MainGoldilocks
