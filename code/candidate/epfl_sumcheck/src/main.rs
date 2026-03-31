use std::io::{self, Read};
use ark_ff::{Field, PrimeField, Zero};
use ark_ff::fields::Fp64;
use efficient_sumcheck::{multilinear_sumcheck, Sumcheck};
use efficient_sumcheck::transcript::SanityTranscript;
use ark_std::rand::SeedableRng;
use ark_std::rand::rngs::StdRng;