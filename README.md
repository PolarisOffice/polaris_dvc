# polaris_dvc

HWPX(OWPML) 문서의 **형식·구조·스키마·무결성**을 한 번에 검증하는 순수 Rust 툴체인이에요. macOS · Linux · Windows · 브라우저(WASM) 어디서든 같은 코드 경로로 빌드됩니다.

🌐 브라우저 데모: <https://polarisoffice.github.io/polaris_dvc/>

## 배경

[rHwp](https://github.com/edwardkim/rhwp) 와 그 주변 논의에서 영감을 받아 시작한 프로젝트입니다. HWPX 는 공개 명세가 있지만, 실제 파일이 명세를 얼마나 따르는지 판정해주는 공개 레퍼런스 구현은 아직 없어요. 한컴 편집기는 명세에서 벗어난 값을 자체 보정해 열어주기 때문에 문서를 여는 쪽에서는 문제가 보이지 않고, 후발 구현체는 "내 파서가 틀린 건지, 파일이 명세와 다른 건지" 구분하기 어렵습니다 ([rHwp #188](https://github.com/edwardkim/rhwp/discussions/188)).

처음에는 [한컴 DVC](https://github.com/hancom-io/dvc) (Windows 전용 C++ DLL) 의 Rust 포팅을 목표로 삼았습니다. 작업을 진행하면서 한컴 DVC 가 다루는 영역은 "규칙 적합성" 한 축이고, 의존 라이브러리의 HWPX 커버리지 한계로 범용 레퍼런스로 쓰기에는 제약이 있다는 걸 확인했어요. 그래서 **한컴 DVC 호환을 핵심 기능 중 하나로 유지하면서, KS X 6101 표준 전체를 커버하는 종합 검증 도구**로 범위를 넓혔습니다.

## 검증의 네 축

`polaris_dvc` 는 HWPX 문서를 네 가지 관점에서 독립적으로 검사합니다. 위반은 JID(error code) 블록으로 구분되어 출력되고요.

| 축 | JID | 내용 |
|---|---|---|
| **규칙 적합성** | 1000–7999 | 규칙 spec JSON 에 정의된 폰트·크기·스타일 허용 범위. **한컴 DVC 와 호환**. |
| **구조 무결성** | 11000–11999 | `charPrIDRef` ↔ `<charPr>` 등의 cross-reference, lineseg 배열, manifest ↔ BinData 일관성. |
| **컨테이너 건전성** | 12000–12999 | ZIP mimetype 위치·압축 방식, 필수 entry, 금지 extras(`__MACOSX/` 등). |
| **스키마 적합성** | 13000–13999 | KS X 6101 XSD 대비 XML 구조·속성·enum. |

축별 상세 설계는 [`docs/hwpx-validation-scope.md`](docs/hwpx-validation-scope.md) 에 있습니다.

## 워크스페이스

```
crates/
├── polaris-dvc-core/     규칙 엔진, 에러코드, 출력 모델
├── polaris-dvc-hwpx/     HWPX(OWPML) 순수 Rust 파서
├── polaris-dvc-schema/   KS X 6101 XSD 기반 스키마 검증기
├── polaris-dvc-format/   포맷 감지 + HWP5 확장 슬롯
├── polaris-dvc-cli/      CLI 바이너리 (polaris-dvc)
└── polaris-dvc-wasm/     wasm-bindgen 래퍼
```

## 빌드

```sh
cargo build --workspace
cargo test  --workspace --exclude polaris-dvc-wasm

# WASM
wasm-pack build crates/polaris-dvc-wasm --target web
```

## CLI 사용법

```sh
cargo run -p polaris-dvc-cli -- [OPTIONS] <HWPX_FILE>
```

### 입력

| 플래그 | 동작 |
|---|---|
| `<HWPX_FILE>` | 검증할 HWPX 파일 경로. `-` 를 주면 stdin 에서 바이트를 읽습니다. |
| `-t <SPEC>` | 규칙 spec JSON 파일 경로. 실제 규칙 값(예: `"fontsize": { "min": 10, "max": 12 }`)이 들어 있는 파일입니다. 작성법은 [`docs/spec-authoring.md`](docs/spec-authoring.md) 참고. |

### 출력

| 플래그 | 동작 |
|---|---|
| `-j`, `--format=json` | (기본) 한컴 DVC 호환 JSON 배열. 필드명·순서는 한컴 DVC 의 `DVCOutputJson.cpp` 와 동일합니다. |
| `-x`, `--format=xml` | 같은 위반 목록을 attribute-per-field XML 로 출력. polaris 확장 기능이고 `--dvc-strict` 와 같이 쓰면 비활성화됩니다. |
| `--file=<PATH>` | 결과를 파일로 저장. 생략하면 stdout. |

### 검사 모드

| 플래그 | 동작 |
|---|---|
| `-a`, `--all` | (기본) 모든 위반을 수집. |
| `-s`, `--simple` | 첫 위반이 나오면 즉시 중단. |
| `--enable-schema` | 스키마 축(JID 13000+) 활성화. 기본은 off — 생성된 XSD 모델이 실제 HWPX 생태계의 drift(`<hp:linesegarray>` 등) 와 충돌해 오탐이 다수 발생하기 때문이에요. 필요할 때만 켜세요. |
| `--dvc-strict` | 한컴 DVC 가 실제로 구현한 JID 만 출력. 11000/12000/13000 축과 한컴 DVC 의 no-op JID(table margin, bgfill 등)를 결과에서 제외해 한컴 DVC 와 바이트 수준 비교가 가능한 모드입니다. |
| `--output-option=<set>` | 출력에 포함할 조건부 필드를 쉼표로 선택. `d`=default, `o`=outline, `t`=table, `i`=image, `p`=page, `y`=style, `k`=hyperlink. 한컴 DVC 의 7 개 단일-문자 토글(`-d`/`-o`/…) 을 하나로 통합한 형태입니다. |

### Exit code

| 코드 | 의미 |
|---|---|
| `0` | 위반 없음 |
| `1` | 위반 검출 |
| `2` | 사용법 오류 (플래그 조합 불가, 파일 없음 등) |
| `3` | HWPX 파싱 실패 |

한컴 DVC 는 exit code 정책을 문서화하지 않아 polaris 쪽에서 자체 정의했습니다. 세부 대응표는 [`docs/cli-compat.md`](docs/cli-compat.md).

### 예제

```sh
# 기본: 전체 검사, 결과를 stdout 으로
cargo run -p polaris-dvc-cli -- -t my-spec.json doc.hwpx

# 첫 위반에서 중단, 파일로 저장
cargo run -p polaris-dvc-cli -- -s --file=out.json -t my-spec.json doc.hwpx

# 한컴 DVC 바이트 호환 모드
cargo run -p polaris-dvc-cli -- --dvc-strict -t my-spec.json doc.hwpx

# 스키마 축까지 포함 (엄격 OWPML 검증)
cargo run -p polaris-dvc-cli -- --enable-schema -t my-spec.json doc.hwpx

# stdin 입력
cat doc.hwpx | cargo run -p polaris-dvc-cli -- -t my-spec.json -
```

## WASM API

```ts
import init, { validate } from "./polaris_dvc.js";
await init();

const report = validate(hwpxBytes, specObject, {
  stopOnFirst: false,
  dvcStrict: false,
  enableSchema: false,
});
// report.violations: [{ errorCode, errorText, pageNo, ... }]
```

## 상태

`testdata/golden/` 의 회귀 테스트 44 케이스가 엔진 동작의 기준점입니다. 한컴 DVC 와의 parity 는 `--dvc-strict` 모드에서 **출력 모양(shape) 호환** — 같은 JID 집합·같은 JSON/XML 필드 레이아웃 — 수준으로 맞춰 두었어요. 리포에서 한컴 DVC 바이너리를 빌드하거나 배포하지는 않습니다 ([`docs/dvc-parity-handoff.md`](docs/dvc-parity-handoff.md) 참고). 남은 커버리지 작업은 [`docs/parity-roadmap.md`](docs/parity-roadmap.md) 에 정리돼 있어요.

## 문서

- [`docs/spec-authoring.md`](docs/spec-authoring.md) — spec 파일 작성 가이드 (필드별 의미 + 카테고리별 예제)
- [`docs/hwpx-validation-scope.md`](docs/hwpx-validation-scope.md) — 네 검증 축 상세 설계
- [`docs/schema-drift-catalog.md`](docs/schema-drift-catalog.md) — 실전 HWPX 샘플에서 나온 Schema 축 findings 와 판정 (divergence vs codegen bug)
- [`docs/cli-compat.md`](docs/cli-compat.md) — CLI 플래그 표면 전체, 한컴 DVC 와의 대응 관계
- [`docs/jid-registry.md`](docs/jid-registry.md) — JID 레지스트리 재생성 및 엔진 확장 방법
- [`docs/golden-tests.md`](docs/golden-tests.md) — 회귀 테스트 운영
- [`docs/parity-roadmap.md`](docs/parity-roadmap.md) — 남은 parity 작업 우선순위
- [`CLAUDE.md`](CLAUDE.md) — AI 에이전트 및 기여자용 작업 노트

## 기여

PR · 이슈 모두 환영합니다. 기여 전에 [`CONTRIBUTING.md`](CONTRIBUTING.md) 와 [`SECURITY.md`](SECURITY.md) 를 한 번 훑어봐 주세요.

## 라이선스

Apache License 2.0.

이 프로젝트는 [한컴 DVC (`hancom-io/dvc`)](https://github.com/hancom-io/dvc) 의 규칙 JSON 스키마, JID 에러코드 체계, 출력 JSON 포맷을 참조해 호환 되도록 Rust 로 재구현한 것입니다. 원저작권은 한컴에 있고 polaris_dvc 는 재구현체로서 그 규격을 **참조**했을 뿐 한컴 DVC 의 컴파일된 코드를 배포하지는 않아요. 자세한 저작권 귀속은 [`NOTICE`](NOTICE), KS X 6101 표준 자료 처리 방침은 [`standards/README.md`](standards/README.md) 를 참고하세요.
