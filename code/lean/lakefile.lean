import Lake
open Lake DSL

package "sumcheck" where
  version := v!"0.1.0"

require sumcheck from git "https://github.com/z-tech/sumcheck-lean4" @ "main"

lean_exe sumcheck where
  root := `Main

lean_exe sumcheck_m31 where
  root := `MainM31

lean_exe sumcheck_babybear where
  root := `MainBabyBear

lean_exe sumcheck_koalabear where
  root := `MainKoalaBear

lean_exe sumcheck_goldilocks where
  root := `MainGoldilocks
