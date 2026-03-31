#import "@preview/charged-ieee:0.1.4": ieee

#show: ieee.with(
  title: [Differential Testing of Cryptographic Implementations Against a Formally Verified Lean Reference],
  abstract: [
    The process of scientific writing is often tangled up with the intricacies of typesetting, leading to frustration and wasted time for researchers. In this paper, we introduce Typst, a new typesetting system designed specifically for scientific writing. Typst untangles the typesetting process, allowing researchers to compose papers faster. In a series of experiments we demonstrate that Typst offers several advantages, including faster document creation, simplified syntax, and increased ease-of-use.
  ],
  authors: (
    (
      name: "Martin Haug",
      department: [Co-Founder],
      organization: [Typst GmbH],
      location: [Berlin, Germany],
      email: "haug@typst.app"
    ),
    (
      name: "Laurenz Mädje",
      department: [Co-Founder],
      organization: [Typst GmbH],
      location: [Berlin, Germany],
      email: "maedje@typst.app"
    ),
  ),
  index-terms: ("Scientific writing", "Typesetting", "Document creation", "Syntax"),
  bibliography: bibliography("refs.bib"),
  figure-supplement: [Fig.],
)

= Introduction
In our digitalized world, cryptographic can be found everywhere, they enable security (or try to) in our everyday online presence and create trust during interractions with other parties. 

== Paper overview
In this paper we are going to verify an implementations of the checksum protocol by differential testing against a verified lean 4 reference. 

In this paper we introduce Typst, a new typesetting system designed to streamline the scientific writing process and provide researchers with a fast, efficient, and easy-to-use alternative to existing systems. Our goal is to shake up the status quo and offer researchers a better way to approach scientific writing.

By leveraging advanced algorithms and a user-friendly interface, Typst offers several advantages over existing typesetting systems, including faster document creation, simplified syntax, and increased ease-of-use.

To demonstrate the potential of Typst, we conducted a series of experiments comparing it to other popular typesetting systems, including LaTeX. Our findings suggest that Typst offers several benefits for scientific writing, particularly for novice users who may struggle with the complexities of LaTeX. Additionally, we demonstrate that Typst offers advanced features for experienced users, allowing for greater customization and flexibility in document creation.

Overall, we believe that Typst represents a significant step forward in the field of scientific writing and typesetting, providing researchers with a valuable tool to streamline their workflow and focus on what really matters: their research. In the following sections, we will introduce Typst in more detail and provide evidence for its superiority over other typesetting systems in a variety of scenarios.

= Methods <sec:methods>

#lorem(240)
= Results
= Discussion
#lorem(240)
= Notes
== Dowloaded the Lean toolchain
Installed the Lean toolchain using Elan.
This gave us: lean and lake
- Lean compiles the code and Lake builds the project.
== Step 2: Clone the Repository

Clone the formal Sumcheck specification and inspect repository contents

== Step 3: Build the Project

Compile the project and dependencies using Lake.

== Step 5: Identify the Protocol Entry Point
Inside Sumcheck/Src/Transcript.lean the key function is:

generate_honest_transcript

This function computes the full Sumcheck protocol transcript:

Transcript
- round_polys
- challenges
- claims

This will act as the formal oracle.

== Step 6: Created an Executable

Modify lakefile.lean to add an executable

== Step 7: Implement a Minimal Main Program
add a full verification step by step to the Main.lean program

== Constructed a multivariate polynomial inside Main.Lean