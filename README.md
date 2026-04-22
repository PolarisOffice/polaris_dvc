# polaris_rhwpdvc

**HWPX 종합 검증 툴** — 규칙·구조·스키마·무결성을 한 번에 검사하는 OWPML conformance suite. 한컴 DVC 와 호환되면서 그 위에 KS X 6101 표준 스키마 검증, ZIP/컨테이너 무결성, cross-reference 정합성까지 쌓아 **HWPX 파일의 종합 품질을 판정**한다. **macOS · Windows · Web(WASM)** 전부 지원.

🌐 **브라우저 데모**: <https://miles-hs-lee.github.io/polaris_rhwpdvc/> — HWPX 파일을 드래그&드롭하면 바로 검증 결과를 확인할 수 있다.

## 왜 만드는가

이 프로젝트는 [rHwp](https://github.com/edwardkim/rhwp) 와 그 주변 논의에서 영감을 받았다.

HWPX(OWPML) 는 공개된 명세가 있지만, 실제 파일 생태계에서는 **명세 준수 여부를 판단하는 기준 구현이 없다**는 문제가 있다. 한컴 뷰어·편집기는 파일을 열 때 명세에서 벗어난 값이 있더라도 자체적으로 보정해 렌더링하기 때문에, 사용자 입장에서는 파일이 정상으로 보인다. 그러나 명세를 신뢰하는 후발 구현체는 그 파일을 그대로 처리했을 때 결과가 달라질 수 있고, 이때 "내 구현이 틀린 것인지, 파일이 명세와 다른 것인지" 판단할 근거가 없다. ([rHwp discussions #188](https://github.com/edwardkim/rhwp/discussions/188) 참고)

이런 상황에서 제3의 구현체가 "한컴 출력과 얼마나 비슷한가"를 유일한 정합성 기준으로 삼으면, 명세와 실제 파일 사이의 간극이 계속 쌓이고 드러나지 않는다. polaris-rhwpdvc 는 그 간극을 드러내고 좁히기 위해 두 가지를 함께 제공하는 것을 목표로 한다.

- **Spec 모드**: OWPML 명세를 엄격히 기준으로 삼아 검증
- **Compat 모드**: 한컴 DVC 와 동일한 출력을 기준으로 삼아 호환성 확인

두 모드를 나란히 운영함으로써, 파일이 "명세상 올바른가"와 "현재 DVC 와 호환되는가"를 동시에 파악할 수 있게 한다.

## 목표 — 검증의 네 축

polaris 는 HWPX 파일을 **여러 각도에서 독립적으로** 검사한다. 각 축은 별도 JID 블록에서
수집되며, 출력 JSON 에서 카테고리별로 분류된다.

1. **규칙 적합성** (Rule conformance) — 사용자가 JSON 으로 정의한 규칙 spec 이
   허용하는 폰트·크기·스타일 범위 안에 문서가 들어가는지. 한컴 업스트림
   [DVC](https://github.com/hancom-io/dvc) 와 호환 (JID 1000~7999). `--dvc-strict` 플래그로
   업스트림이 실제 구현한 JID 만 emit 하는 바이트-호환 모드 운영.
2. **구조 무결성** (Integrity) — cross-reference 일관성 (`charPrIDRef` ↔ `<charPr>`,
   `borderFillIDRef` ↔ `<borderFill>` 등), lineseg 배열 일치, ZIP manifest ↔ BinData sync
   (JID 11000~11999). upstream DVC 에는 없고 polaris 가 추가.
3. **컨테이너 건전성** (Container) — ZIP 수준의 well-formedness: mimetype 위치·압축 방식,
   필수 entry 존재, CRC, 금지 extras (`__MACOSX/` 등) (JID 12000~12999).
4. **스키마 적합성** (Schema) — **KS X 6101** 표준 XSD 대비 각 내부 XML 의 구조·속성·enum
   적합성 검증 (JID 13000~13999). OWPML 표준을 권위 있는 reference 로 삼는다.

네 축을 모두 지원하되 **외부 의존 없는 순수 Rust 구현**. macOS · Windows · `wasm32-unknown-unknown`
전부 동일 코드 경로로 빌드된다.

## 왜 네 축 전부인가

실제 HWPX 생태계는 단일 관점으로 판정할 수 없다:

- **규칙만**으로 검사하면 "이 파일에 특정 폰트가 있는가" 만 답하고, 파일 자체가
  구조적으로 깨졌는지는 모른다 (예: 참조된 ID 가 선언되지 않음)
- **스키마만**으로 검사하면 XML 은 통과해도 cross-ref 정합성·LLM 이 생성한
  허술한 ZIP 구조는 놓친다
- **컨테이너만**으로 검사하면 XML 내용이 명세를 벗어나는지 모른다

polaris 는 네 축 **모두 emit**, 출력 JSON 은 기존 DVC 포맷 유지하면서 카테고리
필드로 구분. 사용자가 원하는 축만 `--only=container,schema` 식으로 필터링 가능.

## 왜 만드는가

이 프로젝트는 [rHwp](https://github.com/edwardkim/rhwp) 와 그 주변 논의에서 영감을 받았다.

HWPX(OWPML) 는 공개된 명세가 있지만, 실제 파일 생태계에서는 **명세 준수 여부를 판단하는 기준 구현이 없다**. 한컴 뷰어·편집기는 파일을 열 때 명세에서 벗어난 값이 있더라도 자체적으로 보정해 렌더링하기 때문에, 사용자 입장에서는 파일이 정상으로 보인다. 그러나 명세를 신뢰하는 후발 구현체는 그 파일을 그대로 처리했을 때 결과가 달라질 수 있고, 이때 "내 구현이 틀린 것인지, 파일이 명세와 다른 것인지" 판단할 근거가 없다. ([rHwp discussions #188](https://github.com/edwardkim/rhwp/discussions/188) 참고)

한편 [한컴 DVC 업스트림](https://github.com/hancom-io/dvc) 은 Windows 전용 C++ DLL 로
좁은 rule 적합성 검증만 수행하고, 자기 샘플 HWPX 조차 crash 하는 수준이라 공개 레퍼런스
로는 불안정하다 (상세는 [`docs/dvc-parity-handoff.md`](docs/dvc-parity-handoff.md)).

polaris-rhwpdvc 는 그 공백을 메워 **KS X 6101 표준 기준의 공개 레퍼런스 체커** 가
되는 것을 목표로 한다. 규칙·구조·스키마·컨테이너 네 축에서 동시에 검사하여 "이
HWPX 가 명세를 어디서 어떻게 벗어났는가" 를 투명하게 드러낸다. DVC 호환은 그 안의
하위 기능 중 하나.

## 호환 매트릭스

| 항목 | 상태 |
|---|---|
| DVC 규칙 JSON 스키마 (JID 1000-7999) | ✅ 업스트림 키 체계 호환 |
| DVC errorCode | ✅ 217 개 JID 전부 registry, drift test |
| DVC 출력 JSON 필드·순서 | ✅ `CharIDRef`, `ParaPrIDRef`, `errorText`, … 동일 |
| 구조 무결성 (JID 11000-11999) | 🚧 확장 중 — cross-ref / lineseg / BinData sync |
| 컨테이너 건전성 (JID 12000-12999) | 🚧 Phase 2 |
| KS X 6101 스키마 적합성 (JID 13000-13999) | 🚧 Phase 3 (`polaris-rhwpdvc-schema` crate) |
| HWPX(OWPML) 순수 Rust 파서 | ✅ 외부 libhwp 의존 없음 |
| HWP 5.0 바이너리 | ❌ 감지만, 파싱 미구현 (후속 crate) |
| 플랫폼 | ✅ Linux · macOS · `wasm32-unknown-unknown` |

상세 범위 설명: [`docs/hwpx-validation-scope.md`](docs/hwpx-validation-scope.md).

## 워크스페이스

```
crates/
├── polaris-rhwpdvc-core/     규칙·에러코드·엔진·출력 모델
├── polaris-rhwpdvc-hwpx/     HWPX(OWPML) 순수 Rust 파서
├── polaris-rhwpdvc-format/   포맷 감지 + DocumentParser trait (HWP5 확장 슬롯)
├── polaris-rhwpdvc-cli/      DVC 호환 CLI (polaris-rhwpdvc)
└── polaris-rhwpdvc-wasm/     wasm-bindgen 래퍼
```

## 빌드

```sh
cargo build --workspace
cargo test  --workspace
```

### CLI

**스펙 파일 ≠ 스키마 파일**. 업스트림 `sample/jsonFullSpec.json` (우리 리포에는
`schemas/jsonFullSpec.json` 으로 복제) 은 **모든 가능한 필드를 나열한 JSON Schema
레퍼런스** — `"fontsize": { "type": "number" }` 같은 메타 기술. 실제 validation 에 쓰려면
거기서 원하는 필드만 뽑아 구체적 값으로 채운 spec 파일을 따로 만들어야 한다. 업스트림
README 의 Demo 도 이런 실제 spec 파일 (`sample/test.json`) 을 넘겨 사용한다.

기본 사용:

```sh
# 실제 spec 파일 하나 (업스트림 sample/test.json 복제본) 로 검증
cargo run -p polaris-rhwpdvc-cli -- \
    -t third_party/dvc-upstream/sample/test.json path/to/document.hwpx

# 또는 우리 golden fixture 중 하나 (간단한 예)
cargo run -p polaris-rhwpdvc-cli -- \
    -t testdata/golden/01_clean/spec.json testdata/golden/01_clean/doc.hwpx

# 파일로 저장 + 첫 오류에서 중단 + DVC.exe 바이트 동일성 모드
cargo run -p polaris-rhwpdvc-cli -- \
    -j --file=out.json -s --dvc-strict \
    -t my-spec.json path/to/document.hwpx

# HWPX 를 stdin 으로 받기 (파이프라인)
cat doc.hwpx | cargo run -p polaris-rhwpdvc-cli -- \
    -j -t my-spec.json -
```

스키마 파일 (`schemas/jsonFullSpec.json`) 을 `-t` 에 넘기는 것은 구조적으로 가능하지만
의미상 맞지 않는다 — 거기 적힌 `"type": "number"` 같은 메타 필드를 검증 규칙으로 해석하려
들면 업스트림은 crash 또는 의도와 다른 동작, polaris 는 파싱 단계에서 유사한 field 불일치를
일으킨다.

**업스트림 DVC 와의 플래그 매핑**: 단순히 "동일" 은 아니고, 실제 호환 정도와 의도적 차이가
있다. 업스트림 `CommandParser.cpp` 기준:

| polaris | 업스트림 | 동작 |
|---|---|---|
| `-j`, `-x` | `-j`, `-x` | 출력 형식. 단 `-x` 는 업스트림이 NotYet 리턴, polaris 는 Extended 프로파일에서 실제 XML 출력 |
| `--file=PATH` | `--file=PATH` | 파일로 저장 |
| `-s`, `-a` | `-s`, `-a` | 첫 오류 중단 / 전체 검사 (기본) |
| **`-t SPEC`** | 다름 | polaris 는 **스펙 파일 경로**. 업스트림 `-t` 는 `OutputOption::Table` 토글 (조건부 필드 축소). 의도적 divergence — 자세한 이유는 `docs/cli-compat.md` |
| `--output-option=<set>` | `-d`/`-o`/`-t`/`-i`/`-p`/`-y`/`-k` | 업스트림의 7 개 단일-문자 토글을 하나의 long flag 로 통합 |
| `--dvc-strict` | 없음 | polaris 전용. 업스트림이 실제 구현한 JID 만 출력 |

업스트림은 스펙·HWPX 를 positional args 로만 받는다 (플래그 자리와 무관). 업스트림 README 가
Demo 예제로 싣는 `ExampleWindows.exe -j --file=... -s -t test.json "005_busan.hwpx"` 의 `-t` 는
**스펙 지정이 아니라** `OutputOption::Table` 토글이고, `test.json` / `005_busan.hwpx` 는 그냥
positional 이다.

전체 플래그 표, exit code 정책, 예제 대응표는 [`docs/cli-compat.md`](docs/cli-compat.md) 참고.

### WASM

```sh
wasm-pack build crates/polaris-rhwpdvc-wasm --target web
wasm-pack build crates/polaris-rhwpdvc-wasm --target nodejs
```

## 상태

현재는 `testdata/golden/` 케이스 기준의 회귀만 보장된다. "DVC 와 동일 출력"의 바이트-정합성은 Windows 환경에서 실제 DVC.exe 결과와 대조해 검증하는 중이며, 그 단계와 남은 parity 작업은 아래 문서에 정리돼 있다.

## 문서

- [docs/cli-compat.md](docs/cli-compat.md) — 업스트림 CLI 플래그 표면과의 대응·차이, exit code 정책
- [docs/golden-tests.md](docs/golden-tests.md) — DVC parity 회귀 테스트 운영 방법
- [docs/jid-registry.md](docs/jid-registry.md) — JID 전체 레지스트리 재생성 및 엔진 확장 가이드
- [docs/dvc-parity-status.md](docs/dvc-parity-status.md) — CI에서 업스트림 DVC 빌드 시도 기록
- [docs/dvc-parity-handoff.md](docs/dvc-parity-handoff.md) — DVC.exe 바이트 parity 달성 시도·실패 이력·다음 단계
- [docs/windows-parity-howto.md](docs/windows-parity-howto.md) — Windows PC 에서 DVC.exe 실행해 `expected.json` 생성·검증하는 절차
- [docs/utm-windows-setup.md](docs/utm-windows-setup.md) — Apple Silicon Mac 에서 UTM + Windows 11 ARM VM 으로 Windows parity 환경 구축
- [docs/parity-roadmap.md](docs/parity-roadmap.md) — 기능 parity 우선순위와 남은 작업

## 라이선스

Apache-2.0. 업스트림 DVC 저작권 귀속은 [`NOTICE`](NOTICE) 참고.
