# Detailed Meeting Summaries and Manual Playback — Proposed Design

Status: approved for Native sequential implementation on 2026-10-23. The user
also authorised additional DeepSeek calls on the unchanged voice_214468.aac
transcript and opening the matching Preview after acceptance. No push/release.

## Intent

Help a reader retain the detail needed to consider a meeting before deciding
what to do next. They should not have to invent a prompt to get that detail,
or hear audio merely because they opened a recording to read its summary.

The user requested detailed meeting prompts, a simpler summary-options screen,
no automatic audio/video playback, and removal of the visible `5` captions
below the seek icons. The proposal retains the five-second seek action; this
interpretation should be corrected before implementation if unwanted.

## Approach and scope

Three approaches were considered before approval:

1. Add only a detailed-prompt example: smallest UI change, but chunk notes may
   already have discarded details before the final prompt is applied.
2. Keep the three formats and add a meeting-detail preference throughout
   generation and reading: recommended. It addresses early detail loss without
   adding another format or replacing the working long-recording pipeline.
3. Add a fourth format or a multi-pass evidence system: more duplicated format
   handling or processing complexity; unnecessary for this request.

The user approved approach2 and its implementation plan. Execution stays
Native and sequential; no subagent delegation or publication is authorised.

## Summary options

- Keep `meeting`, `key_points`, `custom` intact.
- Add `summary.meeting_detail`: `standard` or `detailed`, default `standard`.
  Only meeting mode consumes it. Changing formats preserves this preference
  without forcing it onto key points or a custom prompt.
- Save it together with format/instructions using the existing serialized,
  save-before-commit settings transaction. Missing/unknown values fall back to
  standard without resetting unrelated settings or rewriting old caches.
- Each new result records `meeting_detail`. Reader badges and layout follow
  the saved result, not the preference for the next run. Legacy caches remain
  standard and are not regenerated automatically.
- Format/detail changes never rewrite the user's text. Examples require an
  explicit apply action; replacing nonempty text asks for confirmation.
- Preserve the existing global scope: saved options apply to future runs for
  all files. Saving or viewing examples never calls AI or alters a saved result.

## Proposed options screen

Rename the dialog to “ตั้งค่าการสรุป”. Use the existing modal and theme/font
system, with a wider body on desktop and one column on narrow windows.

1. “รูปแบบสรุป”: three selectable cards with short explanations —
   “สรุปประชุม”, “จับประเด็นสำคัญ”, “เขียน prompt เอง”.
2. Meeting only: “ความละเอียด” with “มาตรฐาน” and “ละเอียด”. Detailed copy:
   “เก็บบริบท เหตุผล ทางเลือก และเงื่อนไขสำคัญ เพื่ออ่านพิจารณาต่อ”.
3. A compact “สิ่งที่จะได้” outline of supported output sections. This is
   static guidance, not an AI-generated preview of the open file.
4. An initially collapsed “เน้นเรื่องเพิ่มเติม / ดูคำสั่งที่จะส่งให้ AI”.
   Custom mode exposes its editor immediately. A chat-proposed prompt also
   opens/focuses that editor. Existing custom text stays editable and intact.
5. A footer that remains reachable while the body scrolls: Cancel and
   “บันทึกการตั้งค่า”. Explain the future-run/global scope and cloud usage.
   Do not introduce an implicit Save-and-run action in this iteration.

Use native radio-group semantics with arrow keys, visible focus and labels.
Tab/Shift-Tab stay inside the modal; Escape/Cancel restore the opener. Disabled
saving and error states retain the user's draft. No third-party UI dependency.

## Detailed prompt contract

The effective built-in instruction, localised to output language, is:

> สรุปการประชุมโดยละเอียดเพื่อใช้ประกอบการพิจารณาขั้นต่อไป ไม่ใช่การถอดคำพูดซ้ำทั้งหมด
> เริ่มด้วยภาพรวม แล้วจัดประเด็นที่คุยกันตามหัวข้อ รักษาบริบท ข้อเสนอของผู้พูด
> เหตุผล ทางเลือก ตัวอย่าง ตัวเลข และเงื่อนไขสำคัญที่มีในต้นฉบับ
> แยกข้อเสนอออกจากสิ่งที่ยืนยันหรือตกลงแล้ว ระบุงาน ผู้รับผิดชอบ และกำหนดส่งเฉพาะที่กล่าวไว้
> เรื่องที่ยังค้างต้องเป็นเรื่องที่ผู้พูดยังไม่ตอบหรือเลื่อนไว้จริง ไม่สร้างเรื่องค้างจากข้อมูลที่ไม่ได้พูด
> หากข้อมูลขัดกันและยังไม่คลี่คลาย ให้ระบุความขัดแย้งพร้อมอ้างอิงทั้งสองส่วน ไม่ตัดสินแทน
> คำถอดเสียงที่ไม่ชัดให้ระบุความไม่แน่นอนหรือไม่ยืนยันคำนั้น ห้ามเดาชื่อหรือหน้าที่เพิ่ม
> รวมข้อความซ้ำได้ แต่ห้ามตัดเหตุผลหรือเงื่อนไขจนความหมายเปลี่ยน
> อ้างอิงแหล่งข้อมูลที่รองรับแต่ละประเด็นเพื่อให้กลับไปฟังได้
> ไม่จำกัดจำนวนประเด็นตายตัว และไม่เติมเนื้อหาเพื่อให้ครบหัวข้อ

Preserve the current five supported meeting sections. This does not override
source-fidelity rules, inject external knowledge, promise a verbatim account,
or certify every AI-written claim.

Apply detail preservation to chunk notes, note reduction, final composition
and bounded review/repair reminders. Do not add Evidence Ledger or another
semantic approval/retry stage. Retain actual-size guards, source-ID checks,
bounded reduction/recovery, cancellation and atomic cache preservation.

Initial implementation keeps the existing provider output ceilings/retry
counts. Detail is a coverage/focus policy, not an unlimited answer size. If the
real detailed run cannot retain material reasons/conditions within those
limits, report that acceptance failure; do not silently label a condensed or
truncated answer “detailed” or raise limits until a lucky run passes.

Detailed meeting results show discussion expanded and in source section order
(overview, discussion, decisions, actions, open issues). Standard meeting
results keep the current outcome-first/folded-discussion presentation. The
reader may collapse sections, but details are not hidden by default. Custom
and key-point layouts remain unchanged. No redesign of chat/library is included.

## Playback and transport

- Loading an audio/video file through Open, folder import, drop, library,
  queue selection or a file change leaves it paused, including restored positions.
- Set native mpv pause before `loadfile`, not after it: no brief audio burst
  while replacing a previously playing file. Loading paused must not retain
  a playback-only sleep-prevention assertion.
- Explicit Play/Space starts playback. Opening a player panel or a summary,
  saving options, generating a summary or seeking never starts paused playback.
  Seeking during explicit playback keeps playback running.
- Do not add new EOF/queue autoplay behavior or reinterpret the unrelated
  `auto_play_next` preference in this task. Existing manual queue navigation
  remains available; a newly loaded item waits for Play.
- Remove only the visible seek-button `5` spans in `ReadingTransport.vue`.
  Retain minus/plus5-second behavior, boundary handling, tooltip and accessible
  Thai/English names. Keep time readout, keyboard shortcuts and speed labels.

## Acceptance

All changes remain local. Test backward compatibility, source-preserving long
generation, cancellation/failure cache retention, narrow/short windows,
keyboard focus, and native no-autoplay/resume behavior. A real detailed run on
the unchanged `voice_214468.aac` transcript requires explicit execution/API
authorisation; none is made during planning. Compare reasons/conditions and
selected citations with the source, not just output length or clock membership.
No source audio, existing summary/transcript cache or production key is replaced.
