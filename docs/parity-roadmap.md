# DVC 기능 Parity 로드맵

현재 polaris는 업스트림 `sample/test.json`이 쓰는 10개 카테고리 전부를
커버하고 있고, `JsonModel.h`의 217개 JID 전부를 `jid_registry.rs`에
상수로 매핑해 두었다. 진짜 "DVC parity"라고 부르려면 남은 것들이 있다.

아래 순위는 **실무 영향도 × 구현 난이도** 기준이다. 각 항목은 현재
상태·예상 작업량·관련 파일까지 명시했다.

---

## P0 — Parity를 주장하려면 반드시

### 1. DVC.exe 바이트-정합성 검증
**왜**: 모든 parity 작업의 "pass" 기준선. 없으면 우리가 "구현했다"고
말하는 것들도 업스트림과 다를 수 있다.

**할 일**:
- Windows PC에서 `scripts\parity-windows.ps1 -WriteExpected` 실행
- 바뀐 `testdata/golden/*/expected.json` 커밋 → polaris 엔진이 실제
  DVC.exe 출력을 따라가도록 수정

**현재**: 스크립트 준비 완료 (`scripts/parity-windows.ps1`), 로드맵
작성자 환경에서 미실행.

**예상 작업량**: Windows 환경에서 초기 1~2 커밋, 이후 이터레이션 N회.

### 2. Table 카테고리 전체 확장 — ⏳ 진행 중
**왜**: 한국 공공문서의 절대 다수가 표를 가짐. 현재 우린 border +
`table-in-table` 2개만 본다. 업스트림 `JID_TABLE_*`은 56개.

**할 일**:
- `jid_registry.rs`에 이미 상수는 있음 (3001~3099)
- 파서: `<hp:tbl>`의 `sz`, `pos`, `margin`, `cellMargin`, `caption`,
  `bgFill`, `effect` 등 파싱
- 엔진: 각 JID에 대응하는 checker 구현
- spec 스키마: `TableSpec`에 `size`, `treatAsChar`, `margin`, `caption`,
  `bgfill` 등 필드 추가

**현재**:
- 구현: border 4방향(type/size/color), table-in-table, size.width/height,
  margin(inMargin) 4방향, treatAsChar
- 미구현: outside margin, bgfill, gradient, picture, effect, caption, pos
  세부, textwrap, rotation, 셀 단위 속성 전부

**예상 작업량**: 5~8 커밋. 한국 공공문서 샘플로 P0-1과 함께 검증하면서
우선 순위 있는 필드부터.

### 3. 범위 기반 spec 값 지원 (`{min, max}` 구조) — ✅ 완료
**왜**: 업스트림 spec은 여러 필드에서 `{ "min": 10, "max": 14 }` 형태를
받는다. 우리는 전부 단일값 등가비교만 지원.

**할 일**:
- 새 타입 `ValueOrRange` (serde-custom deserialize: 숫자 또는
  `{min,max}` 오브젝트 둘 다 수용)
- 영향받는 필드: `charshape.fontsize`, `charshape.ratio`,
  `charshape.spacing`, `parashape.linespacingvalue`,
  `parashape.indent/outdent/spacing-*`, `specialcharacter.minimum/maximum`
  (이미 별개 필드)
- 엔진: 각 비교 분기를 `is_in_range` 로 교체

**현재**: `Range64` 타입으로 단일 값 + `{min, max}` 모두 수용.
CharShape (fontsize/ratio/spacing), ParaShape (linespacingvalue/
spacing-paraup/parabottom/indent/outdent), Table size/margin 모두 적용.

---

## P1 — 표준 준수, 실전 문서 호환

### 4. `errorText` 빈 레코드 드롭 버그 충실 재현 옵션
**왜**: 업스트림 `DVCOutputJson.cpp`의 `||` 오타 때문에 텍스트가 빈
모든 위반(표 속성, 매크로 등)이 실제로는 출력에서 **항상** 드롭된다.
우리는 이 버그를 "고친" 상태로 출력 중이라 매크로 위반이 결과에
보이지만 DVC.exe 결과엔 없을 것. Parity 대조 시 차이가 난다.

**할 일**:
- `OutputOption::DvcBugCompat` 추가 또는 `--dvc-bug-compat` CLI 플래그
- 기본: 우리 수정된 동작 유지 (정보성)
- 플래그 켜면: 업스트림과 동일하게 빈 텍스트 드롭

**현재**: `Report::to_json_value`가 `include_table()`가 true일 때만 빈
텍스트를 보존. 이미 대부분 upstream과 다름.

**예상 작업량**: 1 커밋.

### 5. CharShape 나머지 필드 커버
**왜**: 고급 문서에서 그림자/테두리/언더라인 세부 속성 규정하는 경우 흔함.

**할 일** (JID 블록 1000–1099):
- shadow (type, color, offset x/y) 정확한 비교
- underline (position, shape, color)
- strikeout (shape, color)
- outline type, emboss/engrave boolean
- bg-color, bg-border, bg-pattern
- empty-space, point, kerning
- position, supscript/subscript boolean

**현재**: font, fontsize, bold/italic, underline 존재 유무, strikeout
존재 유무, ratio, spacing. 하위 속성 세부는 미지원.

**예상 작업량**: 3~4 커밋.

### 6. ParaShape 나머지 필드 커버
**할 일** (JID 블록 2000–2099):
- borderFill (paragraph 자체 테두리)
- tab stops
- heading (type, level, outlineShapeIDRef)
- breakSetting (widow/orphan, keepWithNext, …)
- line wrap mode

**예상 작업량**: 2~3 커밋.

### 7. CLI 출력 옵션 플래그 전부 지원
**왜**: 업스트림 `DVCOutputOption` enum은 Default/Table/TableDetail/
Style/Shape/Hyperlink/AllOption 7개. 우린 `OutputOption` enum만 있고
CLI엔 연결 안 됨.

**할 일**:
- `polaris-rhwpdvc-cli`에 `--output-option=table,style,shape,hyperlink,all`
- 각 모드에서 어떤 필드 포함되는지는 이미 `ViolationRecord::to_json_value`
  에 구현

**예상 작업량**: 1 커밋.

### 8. XML 출력 (`-x` / `--format=xml`)
**왜**: 업스트림 지원, 우린 `unimplemented`로 에러 반환.

**할 일**:
- `quick-xml`의 Writer로 `ViolationRecord`를 `<error>` 요소 리스트로
  직렬화
- Element/attribute 이름을 upstream `DVCOutputXml.cpp`와 맞춤 (미확인 —
  소스 확인 필요)

**현재**: CLI에서 `-x` 요청하면 "not yet implemented" 후 exit 2.

**예상 작업량**: 2 커밋 (upstream 포맷 확인 + 구현).

---

## P2 — 고급 문서 지원

### 9. Shape / 임베디드 오브젝트 감지
**왜**: `is_in_shape` 필드. 그림·도형 안에 들어간 텍스트 런의 규칙 예외
처리에 필요.

**할 일**:
- 파서: `<hp:shapeObject>`, `<hp:pic>`, `<hp:drawing>` 스코프 추적
- 엔진: 해당 스코프 내 run은 `is_in_shape=true` 마크

**예상 작업량**: 3 커밋.

### 10. Footnote / Endnote 처리
**왜**: 공공문서 각주 규정이 별도인 경우. 현재 우리 파서는 각주 블록을
통째로 무시.

**할 일**:
- `<hp:footnote>`, `<hp:endnote>` 파싱
- 각주 내 run도 `is_in_shape`와 유사한 플래그로 분류
- 출력에 정상 반영

**예상 작업량**: 2 커밋.

### 11. Hyperlink 상세 검증
**왜**: 현재는 단순 "허용/불허" permission 체크만. Upstream은 URL 패턴,
타겟 등 가능.

**할 일**:
- `<hp:fieldBegin type="HYPERLINK">`의 `command` 속성 (실제 URL) 추출
- spec 확장: `hyperlink.permission` + 추가 필드 (upstream 검토 필요)

**예상 작업량**: 2 커밋.

### 12. 매크로 정의 파싱
**왜**: 현재는 매니페스트의 `.js` 존재 유무만 본다 (upstream `have
MacroInDocument()`와 동치). 하지만 매크로 "사용"까지 본다면 문서 내부
`<hp:script>` 블록 등도 봐야 함.

**예상 작업량**: 1~2 커밋.

---

## P3 — 폴리시

### 13. 페이지·라인 번호 정확도 고도화
업스트림 `FindPageInfo`를 1:1 포팅했지만 실제 복잡한 문서
(`<hp:linesegarray>` 안에 페이지 브레이크가 여러 번 있는 경우 등)에서
정확히 동일한 값이 나오는지 미검증. P0-1 결과로 자연스럽게 드러남.

### 14. Special character 범위 — unicode range 여러 구간
Upstream은 여러 범위 합집합을 지원할 수 있음 (확인 필요). 현재 우린
단일 min/max.

### 15. Numbering/Bullet sub-attribute
현재 numbertype/numbershape만. Upstream에는 `align`, `useInstWidth`,
`autoIndent`, `widthAdjust`, `textOffset`, `charPrIDRef`, `checkable`
등 추가 필드 있음.

---

## 작업 진행 시 가이드라인

1. **항상 JID 레지스트리 먼저 확인**: 구현하려는 속성이
   `jid_registry::JID_*`에 이미 상수로 있으면 그걸 엔진에 alias로 등록
   (`error_codes.rs::jid`). 숫자 하드코딩 금지.
2. **Golden 케이스를 먼저**: 구현 전 `testdata/golden/<case>/`에
   실패하는 새 케이스를 먼저 커밋. TDD.
3. **DVC.exe 대조 주기적으로**: 한 덩어리 끝내면 `parity-windows.ps1
   -WriteExpected` 돌려 우리 출력이 upstream과 여전히 맞는지 확인.
4. **P0 1번 해결 전까지는 `expected.json`이 우리 엔진 출력**: 그게
   진짜 parity인지는 검증 전. P0-1 이후부터 진짜 parity.

---

## 추정 총 작업량

P0 전체: ~10 커밋, 1~2일 집중
P1 전체: ~10 커밋, 1~2일
P2 전체: ~8 커밋, 2~3일
P3 전체: 지속적으로 2~4 커밋씩

→ P0 완료 시점이 "DVC parity 실질적 달성" 이정표. P1까지 가면 실무에서
DVC.exe 대체 가능 수준.
