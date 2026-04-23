# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다. [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/) 포맷을 따르고, 버전은 [SemVer](https://semver.org/lang/ko/) 규칙을 따릅니다. **0.x 대에서는 minor 버전 bump 에 breaking change 가 포함될 수 있습니다**.

## [Unreleased]

## [0.1.0] - 2026-04-24

첫 공개 릴리스. HWPX(OWPML) 문서를 네 가지 관점으로 검증하는 순수 Rust 툴체인.

### Added — 검증 엔진

- **4축 검증 프레임워크** — 규칙 적합성 (JID 1000–7999, 한컴 DVC 호환), 구조 무결성 (11000–11999), 컨테이너 건전성 (12000–12999), 스키마 적합성 (13000–13999). 각 축은 독립적으로 on/off.
- **217 개 JID** 전량을 `third_party/dvc-upstream/Source/JsonModel.h` 에서 자동 생성 (`tools/gen-jids`). 드리프트 감지 테스트가 커밋마다 확인.
- **규칙 카테고리 커버리지**: charshape(font/size/bold/color/shadow/emboss/engrave/supscript/subscript/ratio/spacing/kerning/r-size), parashape(indent/align/linespacing mode+value/horizontal), table(border/size/margin/treatAsChar/bgfill type·facecolor·pattoncolor·pattontype/pos/textpos/fixed), specialcharacter min/max, bullet/outlineshape/paranumbullet numtype+numshape, style/hyperlink/macro 허용 여부, shape/footnote/endnote 스코프 내 위반 추적.
- **범위 spec 값** — 모든 수치 규칙이 단일 값 또는 `{"min": N, "max": M}` 둘 다 허용 (`Range64`).
- **페이지/라인 번호 추적** — 한컴 DVC `OWPMLReader::FindPageInfo` 를 bit-for-bit 포팅.
- **구조 무결성 체크** — `charPrIDRef` ↔ `<charPr>` 등 cross-reference (11001–11003), 빈 `lineSegArray` (11004), ZIP mimetype 위치·압축 (11010–11012), manifest ↔ BinData 3-way 동기화 (11020–11022).
- **스키마 축** — KS X 6101 XSD 에서 생성한 OWPML 모델로 XML 구조·속성·enum 검증. 실전 HWPX 드리프트 때문에 기본 off, `--enable-schema` 로 활성화.

### Added — 출력 / 호환성

- **한컴 DVC JSON 출력 호환** — `DVCOutputJson.cpp` 와 필드명·순서·조건부 field 가 동일. `--dvc-strict` 프로파일에서 업스트림이 실제 검증하는 JID 만 통과.
- **XML 출력** (`-x` / `--format=xml`) — attribute-per-field 포맷, JSON 과 diff 가능한 필드 순서. polaris 확장이며 `--dvc-strict` 와는 조합 불가 (업스트림 미구현 반영).
- **7 개 출력 옵션 단일 플래그** — `--output-option=<set>` 으로 한컴 DVC 의 `-d`/`-o`/`-t`/`-i`/`-p`/`-y`/`-k` 토글 통합 (default/outline/table/image/page/style/hyperlink).
- **Click-to-locate 힌트** — 4축 전부의 위반 레코드가 `ErrorString` + `FileLabel` + `ByteOffset` 를 운반. `--dvc-strict` 에서는 업스트림 바이트 호환을 위해 push 시점에 제거.

### Added — 플랫폼 / 배포

- **CLI 바이너리** `polaris-dvc` — 한컴 DVC 와 동일한 플래그 표면.
- **WASM 바인딩** (`polaris-dvc-wasm`) — 브라우저/Node 양쪽에서 단일 `validate()` 엔트리.
- **브라우저 데모** <https://polarisoffice.github.io/polaris_dvc/> — ZIP 익스플로러, XML 신택스 하이라이팅, 이미지 프리뷰, 클릭-투-이동.
- **macOS · Linux · Windows · `wasm32-unknown-unknown`** 에서 동일 코드 경로.

### Added — 테스트 / 개발 도구

- **44 케이스 골든 회귀 스위트** (`testdata/golden/`) — `doc.hwpx` 바이트, `spec.json`, `expected.json`, `expected.xml` 4 종 삼중체크. `POLARIS_REGEN_FIXTURES=1` 로 재생성.
- **JID 레지스트리 드리프트 테스트** — 커밋마다 업스트림 헤더 파싱해 숫자 차이 감지.
- **스키마 코드젠** (`tools/gen-owpml`) — KS X 6101 XSD 에서 Rust validator 모델 생성.
- **Dependabot** — cargo + github-actions 2축 주간 감시, semver-0.x breaking-change (quick-xml, zip) 는 pin.

### Known limitations

- **스키마 축이 기본 비활성화** — 코드젠 모델이 실전 HWPX 에서 나오는 `<hp:linesegarray>`, `<hp:switch>` 같은 Hancom 확장과 드리프트가 있어 오탐이 나옴. 필요할 때 `--enable-schema` 로 켜고 판정은 [`docs/schema-drift-catalog.md`](docs/schema-drift-catalog.md) 참조.
- **한컴 DVC `DVC.exe` 와의 바이트 parity 는 범위 외** — 이 리포는 업스트림 바이너리를 빌드·배포하지 않음. Parity 는 `--dvc-strict` 출력 shape 수준에서 유지 ([`docs/dvc-parity-handoff.md`](docs/dvc-parity-handoff.md)).
- **HWP5 포맷 미지원** — `polaris-dvc-format` 에 슬롯만 예약되어 있고 HWPX 만 실제 파싱.

### Contributors

[@miles-hs-lee](https://github.com/miles-hs-lee), Claude Opus 4.7

[Unreleased]: https://github.com/PolarisOffice/polaris_dvc/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/PolarisOffice/polaris_dvc/releases/tag/v0.1.0
