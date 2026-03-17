# Campaign: office-skills-refactor
## Original Prompt (Verbatim)

---

Use the campaign architecture to make a campaign for all: { Top Inefficiencies

  1. 3x Code Duplication — BaseSchemaValidator (biggest issue)

  docx/ooxml/scripts/validation/base.py, pptx/ooxml/scripts/validation/base.py, and
  xlsx/ooxml/scripts/validation/base.py are near-identical (~950 lines each). That's ~2,850
  lines of duplicated validation logic. Bug fixes and changes must be applied in 3 places.

  2. Pack/Unpack Scripts Duplicated Too

  docx/ooxml/scripts/{pack,unpack,validate}.py mirror pptx/ooxml/scripts/{pack,unpack,validate}.py.
  Same problem — 6 files where 3 should exist.

  3. html2pptx.js Is a 995-Line Monolith

  It mixes browser automation, 21 validation rules, element parsers, unit conversion helpers, and
  conversion orchestration all in one file. Hard to test any piece of it in isolation.

  4. Fragile Monkeypatch in fill_fillable_fields.py

  Lines 78–103 patch a pypdf bug with no version pin. If pypdf updates, it silently breaks.
  The fix looks obvious in-context but is invisible to anyone reading later.

  5. Unit Conversion Constants Scattered

  PT_PER_PX, PX_PER_IN, EMU_PER_IN are defined ad-hoc in individual scripts instead of one
  shared constants file.

  6. No Centralized Error Code System

  Errors across base.py, fill_fillable_fields.py, and html2pptx.js are inconsistent prose strings —
  some throw, some return, some print. Can't programmatically distinguish error types.

  ---
  Biggest ROI fix: Pull base.py into public/validation/shared/ and update the 3 import paths.
  That alone eliminates ~1,900 lines of duplicated code that currently has to be maintained in
  triplicate. } to fix all of these with the best solution

---

*Captured: 2026-03-16*
*Campaign status: active*
