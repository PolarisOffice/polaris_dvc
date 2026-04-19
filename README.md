# polaris_rhwpdvc

한컴 **DVC**(Document Validation Checker)의 멀티플랫폼 포팅. Windows 전용이던 DVC 검증 파이프라인을 Rust로 다시 구현해 **macOS · Windows · Web(WASM)** 어디서든 동일한 규칙 JSON·errorCode·출력 포맷으로 HWPX 문서를 검증하는 것이 목표다.

🌐 **브라우저 데모**: <https://miles-hs-lee.github.io/polaris_rhwpdvc/> — HWPX 파일을 드래그&드롭하면 바로 검증 결과를 확인할 수 있다.

## 왜 만드는가

이 프로젝트는 [rHwp](https://github.com/edwardkim/rhwp) 와 그 주변 논의에서 영감을 받았다.

HWPX(OWPML) 는 공개된 명세가 있지만, 실제 파일 생태계에서는 **명세 준수 여부를 판단하는 기준 구현이 없다**는 문제가 있다. 한컴 뷰어·편집기는 파일을 열 때 명세에서 벗어난 값이 있더라도 자체적으로 보정해 렌더링하기 때문에, 사용자 입장에서는 파일이 정상으로 보인다. 그러나 명세를 신뢰하는 후발 구현체는 그 파일을 그대로 처리했을 때 결과가 달라질 수 있고, 이때 "내 구현이 틀린 것인지, 파일이 명세와 다른 것인지" 판단할 근거가 없다. ([rHwp discussions #188](https://github.com/edwardkim/rhwp/discussions/188) 참고)

이런 상황에서 제3의 구현체가 "한컴 출력과 얼마나 비슷한가"를 유일한 정합성 기준으로 삼으면, 명세와 실제 파일 사이의 간극이 계속 쌓이고 드러나지 않는다. polaris 는 그 간극을 드러내고 좁히기 위해 두 가지를 함께 제공하는 것을 목표로 한다.

- **Spec 모드**: OWPML 명세를 엄격히 기준으로 삼아 검증
- **Compat 모드**: 한컴 DVC 와 동일한 출력을 기준으로 삼아 호환성 확인

두 모드를 나란히 운영함으로써, 파일이 "명세상 올바른가"와 "현재 DVC 와 호환되는가"를 동시에 파악할 수 있게 한다.

## 1.0 목표

Compat 모드를 먼저 완성하는 단계다. 기존 DVC 자산(규칙 파일, 연동 툴링)을 그대로 쓰면서 플랫폼 제약만 없애는 것이 핵심이다.

- 한컴 [DVC 업스트림](https://github.com/hancom-io/dvc) 과 동일한 규칙 JSON 스키마 해석
- `JID 1000~10999` errorCode 블록 전부 호환
- 출력 JSON 필드·구조 동일 (`CharIDRef`, `ParaPrIDRef`, `PageNo`, `LineNo`, `ErrorCode`, `errorText` …)
- HWPX(OWPML) 순수 Rust 파서, 외부 libhwp 의존 없음
- macOS · Windows · `wasm32-unknown-unknown` 타깃 동시 지원
- 업스트림 CLI 플래그(`-j`, `-x`, `--file=`, `-s`, `-a`, `-t <spec>`) 호환

레거시 HWP 5.0 바이너리 포맷은 `polaris-rhwpdvc-format` 에서 감지만 하고 `Hwp5NotImplemented` 를 반환한다. 별도 crate 로 후속 버전에서 붙인다.

## 2.0 이후 방향

Spec 모드를 중심으로, **이 프로젝트 자체가 HWPX 검증의 Single Source of Truth** 가 되는 것을 목표로 한다. 공개된 OWPML 명세와 이 저장소의 규칙·엔진·테스트 코퍼스가 "무엇이 올바른 HWPX 인가"를 정의하고, 어떤 구현체든 그 기준에 맞춰 자신의 준수 수준을 측정할 수 있는 레퍼런스 구현이 되는 것이 장기 지향점이다.

## 호환 매트릭스

| 항목 | 호환 수준 |
|---|---|
| 규칙 JSON 스키마 | `third_party/dvc-upstream/sample/jsonFullSpec.json` 과 동일 키 체계 |
| errorCode | JID 1000~10999 블록 동일 (`polaris-rhwpdvc-core` `ErrorCode`) |
| 출력 JSON | 필드명·구조 동일 |
| HWPX | Phase 3~ 구현 중 (진행 상황은 `docs/parity-roadmap.md` 참고) |
| HWP 5.0 바이너리 | 감지만, 파싱은 미구현 |
| 플랫폼 | Linux · macOS · `wasm32-unknown-unknown` |

## 워크스페이스

```
crates/
├── polaris-rhwpdvc-core/     규칙·에러코드·엔진·출력 모델
├── polaris-rhwpdvc-hwpx/     HWPX(OWPML) 순수 Rust 파서
├── polaris-rhwpdvc-format/   포맷 감지 + DocumentParser trait (HWP5 확장 슬롯)
├── polaris-rhwpdvc-cli/      DVC 호환 CLI (polaris)
└── polaris-rhwpdvc-wasm/     wasm-bindgen 래퍼
```

## 빌드

```sh
cargo build --workspace
cargo test  --workspace
```

### CLI

```sh
cargo run -p polaris-rhwpdvc-cli -- -j -t schemas/jsonFullSpec.json path/to/document.hwpx
```

플래그는 업스트림 DVC 와 동일하다 (`-j`, `-x`, `--file=`, `-s`, `-a`, `-t <spec>`).

### WASM

```sh
wasm-pack build crates/polaris-rhwpdvc-wasm --target web
wasm-pack build crates/polaris-rhwpdvc-wasm --target nodejs
```

## 상태

현재는 `testdata/golden/` 케이스 기준의 회귀만 보장된다. "DVC 와 동일 출력"의 바이트-정합성은 Windows 환경에서 실제 DVC.exe 결과와 대조해 검증하는 중이며, 그 단계와 남은 parity 작업은 아래 문서에 정리돼 있다.

## 문서

- [docs/golden-tests.md](docs/golden-tests.md) — DVC parity 회귀 테스트 운영 방법
- [docs/jid-registry.md](docs/jid-registry.md) — JID 전체 레지스트리 재생성 및 엔진 확장 가이드
- [docs/dvc-parity-status.md](docs/dvc-parity-status.md) — CI에서 업스트림 DVC 빌드 시도 기록
- [docs/windows-parity-howto.md](docs/windows-parity-howto.md) — Windows PC 에서 DVC.exe 실행해 `expected.json` 생성·검증하는 절차
- [docs/parity-roadmap.md](docs/parity-roadmap.md) — 기능 parity 우선순위와 남은 작업

## 라이선스

Apache-2.0. 업스트림 DVC 저작권 귀속은 [`NOTICE`](NOTICE) 참고.
