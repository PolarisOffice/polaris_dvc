# polaris_rhwpdvc

한컴 **DVC**(Document Validation Checker)의 멀티플랫폼 포팅. Windows 전용이던 DVC 검증 파이프라인을 Rust로 다시 구현해 **macOS · Windows · Web(WASM)** 어디서든 동일한 규칙 JSON·errorCode·출력 포맷으로 HWPX 문서를 검증하는 것이 목표다.

## 왜 만드는가

이 프로젝트는 [rHwp](https://github.com/edwardkim/rhwp) 에서 영감을 받았다. rHwp 의 [issue #185](https://github.com/edwardkim/rhwp/issues/185) 가 지적하듯, 한컴 자체 구현은 공개된 HWPX 표준만 따르지 않는다. 실제 배포된 `.hwpx` 에는 표준 스펙에 없는 비공개 확장이 섞여 있고, 그 결과 한컴이 아닌 구현체는 "공개 스펙만으로는 재현할 수 없는 비표준"을 역공학으로 구현해야 유사한 검증/렌더링 결과를 낼 수 있다.

그런 부담을 표준·검증 레이어부터 분담하기 위해, 한컴 DVC 와 **바이트-정합성** 수준으로 맞춰지는 오픈 구현을 만든다. 1.0 의 범위는 DVC 와 **동일 기능**(규칙 JSON 스키마, errorCode 블록, 출력 JSON 스키마)이고, 그 위에 Windows 에만 묶여있던 환경 제약을 푸는 것이다.

## 1.0 목표

- 한컴 [DVC 업스트림](https://github.com/hancom-io/dvc) 과 동일한 규칙 JSON 스키마 해석
- `JID 1000~10999` errorCode 블록 전부 호환
- 출력 JSON 필드·구조 동일 (`CharIDRef`, `ParaPrIDRef`, `PageNo`, `LineNo`, `ErrorCode`, `errorText` …)
- HWPX(OWPML) 순수 Rust 파서, 외부 libhwp 의존 없음
- Linux · macOS · `wasm32-unknown-unknown` 타깃 동시 지원
- 업스트림 CLI 플래그(`-j`, `-x`, `--file=`, `-s`, `-a`, `-t <spec>`) 호환

레거시 HWP 5.0 바이너리 포맷은 `polaris-rhwpdvc-format` 에서 감지만 하고 `Hwp5NotImplemented` 를 반환한다. 별도 crate 로 후속 버전에서 붙인다.

## 2.0 이후 방향

1.0 이 "한컴 DVC 와 동일 결과"를 보장하는 단계라면, 2.0 이후는 **이 프로젝트 자체가 HWPX 검증의 Single Source of Truth** 가 되는 것을 목표로 한다. 더 이상 한컴 자체 구현을 기준으로 역추적하지 않고, 공개된 HWPX 표준과 이 저장소의 규칙·엔진·테스트 코퍼스가 "무엇이 올바른 HWPX 인가"를 정의한다. 한컴 구현을 포함한 모든 구현체가 이 레퍼런스에 맞추는 방향으로 뒤집는 것이 장기 지향점이다.

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
