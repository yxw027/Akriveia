# Design_Specification
Project to keep track of tasks and issues relating to Design Specification document

See grading rubric for document details

Due July 07, 2019

## To generate Glossary
pdflatex __Requirements_Specification.tex

makeindex -s __Requirements_Specification.ist -o __Requirements_Specification.gls __Requirements_Specification.glo

pdflatex __Requirements_Specification.tex

## To generate Reference
pdflatex __Requirements_Specification.tex

bibtex __Requirements_Specification

pdflatex __Requirements_Specification.tex

pdflatex __Requirements_Specification.tex