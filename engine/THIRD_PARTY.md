# Local translation components

Our stdio adapter is MIT. It runs CTranslate2 4.8.2 (MIT), SentencePiece 0.2.2
(Apache-2.0), NumPy 2.5.3 (BSD and bundled component notices), PyYAML 6.0.3
(MIT), Python 3.12 (PSF license) and PyInstaller 6.22.3 (GPL with its bootloader
distribution exception). Exact distribution license files are collected into
the executable's `licenses/` directory by scripts/build-engine.py. This file
and the dependency inventory are included with those notices.

Windows also includes Intel OpenMP 2025.3.0 (`libiomp5md.dll`). Its SHA-256 was
compared with the Intel-published PyPI wheel and matched exactly; that wheel's
full LICENSE.txt is preserved in `licenses/upstream/`. Optional cuDNN/CUDA
libraries are excluded by the CPU packaging hook. CTranslate2 and SentencePiece
license texts come from their exact upstream version tags (`engine/licenses/`).

Model weights are not bundled in the installer. engine/catalog.json contains
each selected archive's source, version, exact byte size, SHA-256, attribution
and license evidence. Installation preserves original README/metadata and
stores this manifest next to the model. No weights are modified.

Portuguese/English 1.9 packages identify their OPUS-MT source as CC-BY-4.0:
Jörg Tiedemann and Santhosh Thottingal, “OPUS-MT — Building open translation
services for the World”, EAMT 2020, Lisbon. Preserve their attribution.
https://creativecommons.org/licenses/by/4.0/

German/English 1.3 archives contain no individual license text. The Argos
maintainer states that .argosmodel binaries use MIT/CC0 in issue #533:
https://github.com/argosopentech/argos-translate/issues/533
Argos Open Technologies, PJ Finlay:
https://github.com/argosopentech/argos-translate/blob/master/LICENSE

The adapter reads official Argos model data directly with CTranslate2 and
SentencePiece. It does not ship the Argos Python library, Stanza, MiniSBD,
LibreTranslate, Flask, or Torch. Its sentence splitting differs from Argos;
identical translation quality/output is not promised.

## Argos MIT notice

Copyright (c) 2020 Argos Open Technologies, LLC

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
