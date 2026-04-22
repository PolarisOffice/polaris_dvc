# polaris_rhwpdvc

HWPX(OWPML) 문서의 **형식·구조·스키마·무결성**을 한 번에 검증하는 순수 Rust 툴체인. macOS · Linux · Windows · 브라우저(WASM) 모두 같은 코드 경로로 빌드된다.

🌐 브라우저 데모: <https://miles-hs-lee.github.io/polaris_rhwpdvc/>

## 배경

[rHwp](https://github.com/edwardkim/rhwp) 와 그 주변 논의에서 영감을 받아 시작했다. HWPX 는 공개 명세가 있지만, 실제 파일이 명세를 얼마나 따르는지 판정해주는 공개 레퍼런스 구현이 없다. 한컴 편집기는 명세에서 벗어난 값을 자체 보정해 열기 때문에 사용자 입장에서는 문제가 드러나지 않고, 후발 구현체는 "내가 틀린 건지 파일이 틀린 건지" 구분하기 어렵다 ([rHwp #188](https://github.com/edwardkim/rhwp/discussions/188)).

초기엔 [한컴 DVC](https://github.com/hancom-io/dvc) (Windows 전용 C++ DLL) 의 Rust 포팅을 목표로 했지만, DVC 가 다루는 건 "규칙 적합성" 한 축뿐이고 자체 샘플에서도 불안정해 공개 레퍼런스로 쓰기 어렵다는 것이 드러났다. 그래서 **DVC 호환을 하위 기능으로 포함하면서 KS X 6101 표준 전체를 검증하는 종합 툴**로 범위를 넓혔다.

## 검증의 네 축

문서를 서로 다른 관점에서 독립적으로 검사하고, 위반은 JID(error code) 블록으로 구분되어 출력된다.

| 축 | JID | 내용 |
|---|---|---|
| **규칙 적합성** | 1000–7999 | 사용자 spec JSON 에 정의된 폰트·크기·스타일 허용 범위. DVC 호환. |
| **구조 무결성** | 11000–11999 | `charPrIDRef` ↔ `<charPr>` 등 cross-reference, lineseg 배열, manifest ↔ BinData sync. |
| **컨테이너 건전성** | 12000–12999 | ZIP mimetype 위치·압축 방식, 필수 entry, 금지 extras(`__MACOSX/` 등). |
| **스키마 적합성** | 13000–13999 | KS X 6101 XSD 대비 XML 구조·속성·enum. |

축별 상세: [`docs/hwpx-validation-scope.md`](docs/hwpx-validation-scope.md).

## 워크스페이스

```
crates/
├── polaris-rhwpdvc-core/     규칙 엔진, 에러코드, 출력 모델
├── polaris-rhwpdvc-hwpx/     HWPX(OWPML) 순수 Rust 파서
├── polaris-rhwpdvc-schema/   KS X 6101 XSD 기반 스키마 검증기
├── polaris-rhwpdvc-format/   포맷 감지 + HWP5 확장 슬롯
├── polaris-rhwpdvc-cli/      CLI 바이너리 (polaris-rhwpdvc)
└── polaris-rhwpdvc-wasm/     wasm-bindgen 래퍼
```

## 빌드

```sh
cargo build --workspace
cargo test  --workspace --exclude polaris-rhwpdvc-wasm

# WASM
wasm-pack build crates/polaris-rhwpdvc-wasm --target web
```

## CLI 사용법

```sh
cargo run -p polaris-rhwpdvc-cli -- [OPTIONS] <HWPX_FILE>
```

### 입력

| 플래그 | 동작 |
|---|---|
| `<HWPX_FILE>` | 검증할 HWPX 파일 경로. `-` 이면 stdin 으로 바이트 입력. |
| `-t <SPEC>` | 사용자 규칙 spec JSON 파일. `fontsize`, `font.allowlist` 같은 구체적 값을 담은 파일이다. `schemas/jsonFullSpec.json` 은 "사용 가능한 필드 전체를 나열한 메타 레퍼런스"지 실제 spec 이 아니므로 여기에 넘기면 안 된다. |

### 출력

| 플래그 | 동작 |
|---|---|
| `-j`, `--format=json` | (기본) DVC 호환 JSON 배열. 필드명·순서는 업스트림 `DVCOutputJson.cpp` 와 동일. |
| `-x`, `--format=xml` | 같은 위반 목록을 attribute-per-field XML 로. polaris 확장 기능이며 `--dvc-strict` 와 함께 쓰면 비활성화된다. |
| `--file=<PATH>` | 결과를 파일로 저장. 생략 시 stdout. |

### 검사 모드

| 플래그 | 동작 |
|---|---|
| `-a`, `--all` | (기본) 모든 위반을 수집. |
| `-s`, `--simple` | 첫 위반에서 즉시 중단. |
| `--enable-schema` | 스키마 축(JID 13000+) 활성화. 기본은 off — 생성된 XSD 모델이 실제 HWPX 생태계의 drift(`<hp:linesegarray>` 등) 와 충돌해 false positive 가 다수 발생하기 때문. |
| `--dvc-strict` | DVC.exe 가 실제 구현한 JID 만 emit. 11000/12000/13000 축과 upstream no-op JID(table margin, bgfill 등)를 출력에서 제외해 업스트림과 바이트 수준 비교가 가능한 모드. |
| `--output-option=<set>` | 출력에 포함할 조건부 필드를 쉼표로 선택. `d`=default, `o`=outline, `t`=table, `i`=image, `p`=page, `y`=style, `k`=hyperlink. 업스트림 DVC 의 7 개 단일-문자 토글(`-d`/`-o`/…) 을 하나로 통합한 것. |

### Exit code

| 코드 | 의미 |
|---|---|
| `0` | 위반 없음 |
| `1` | 위반 검출 |
| `2` | 사용법 오류 (플래그 조합 불가, 파일 없음 등) |
| `3` | HWPX 파싱 실패 |

업스트림 DVC 는 exit code 정책을 명시하지 않아 polaris 자체 정의. 세부 대응표: [`docs/cli-compat.md`](docs/cli-compat.md).

### 예제

```sh
# 기본: 전체 검사, 결과를 stdout 으로
cargo run -p polaris-rhwpdvc-cli -- -t my-spec.json doc.hwpx

# 첫 위반에서 중단, 파일로 저장
cargo run -p polaris-rhwpdvc-cli -- -s --file=out.json -t my-spec.json doc.hwpx

# DVC.exe 바이트 호환 모드
cargo run -p polaris-rhwpdvc-cli -- --dvc-strict -t my-spec.json doc.hwpx

# 스키마 축까지 포함 (엄격 OWPML 검증)
cargo run -p polaris-rhwpdvc-cli -- --enable-schema -t my-spec.json doc.hwpx

# stdin 입력
cat doc.hwpx | cargo run -p polaris-rhwpdvc-cli -- -t my-spec.json -
```

## WASM API

```ts
import init, { validate } from "./polaris_rhwpdvc.js";
await init();

const report = validate(hwpxBytes, specObject, {
  stopOnFirst: false,
  dvcStrict: false,
  enableSchema: false,
});
// report.violations: [{ errorCode, errorText, pageNo, ... }]
```

## 상태

`testdata/golden/` 회귀 테스트 44 케이스가 엔진의 정답. DVC.exe 와의 바이트 parity 는 Windows 환경에서 개별 검증하며, 남은 작업은 [`docs/parity-roadmap.md`](docs/parity-roadmap.md) 에 정리돼 있다.

## 문서

- [`docs/hwpx-validation-scope.md`](docs/hwpx-validation-scope.md) — 네 검증 축 상세 설계
- [`docs/cli-compat.md`](docs/cli-compat.md) — CLI 플래그 표면 전체, 업스트림 대응·차이
- [`docs/jid-registry.md`](docs/jid-registry.md) — JID 레지스트리 재생성 및 엔진 확장
- [`docs/golden-tests.md`](docs/golden-tests.md) — 회귀 테스트 운영
- [`docs/parity-roadmap.md`](docs/parity-roadmap.md) — 남은 parity 작업 우선순위
- [`CLAUDE.md`](CLAUDE.md) — AI 에이전트 및 기여자용 내부 작업 노트

## 라이선스

Apache-2.0. 업스트림 DVC 저작권 귀속은 [`NOTICE`](NOTICE), KS X 6101 자료 처리 방침은 [`standards/README.md`](standards/README.md) 참고.
