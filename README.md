# polaris_rhwpdvc

[hancom-io/dvc](https://github.com/hancom-io/dvc)의 Rust 포팅. HWPX 문서를 DVC와 동일한 규칙 JSON 스키마·errorCode·출력 포맷으로 검증한다. 기존 DVC 자산을 그대로 쓰면서 Linux·macOS·WASM에서 동작하는 것을 목표로 한다.

## 호환 매트릭스

| 항목 | 호환 수준 |
|---|---|
| 규칙 JSON 스키마 | `third_party/dvc-upstream/sample/jsonFullSpec.json`과 동일 키 체계 |
| errorCode | JID 1000~10999 블록 동일 (`polaris-core` `ErrorCode`) |
| 출력 JSON | 필드명·구조 동일 (`CharIDRef`, `ParaPrIDRef`, `PageNo`, `LineNo`, `ErrorCode`, `errorText` …) |
| HWP 바이너리 | HWPX 구현(Phase 3~). 레거시 HWP 5.0은 `polaris-format`의 dispatch만 열어두고 별도 crate로 추후 |
| 플랫폼 | Linux · macOS · `wasm32-unknown-unknown` |

## 워크스페이스

```
crates/
├── polaris-core/     규칙·에러코드·엔진·출력 모델
├── polaris-hwpx/     HWPX (OWPML) 순수 Rust 파서
├── polaris-format/   포맷 감지 + DocumentParser trait (HWP5 확장 슬롯)
├── polaris-cli/      DVC 호환 CLI (polaris)
└── polaris-wasm/     wasm-bindgen 래퍼
```

## 빌드

```sh
cargo build --workspace
cargo test --workspace
```

### CLI

```sh
cargo run -p polaris-cli -- -j -t schemas/jsonFullSpec.json path/to/document.hwpx
```

플래그는 업스트림 DVC와 동일하다 (`-j`, `-x`, `--file=`, `-s`, `-a`, `-t <spec>`).

### WASM

```sh
wasm-pack build crates/polaris-wasm --target web
wasm-pack build crates/polaris-wasm --target nodejs
```

## 상태

단계별 계획 및 진행 상황은 `/docs/` 및 커밋 로그를 참고한다. 초기 버전은 HWPX만 지원하고, HWP 5.0은 `polaris-format::DocumentFormat::Hwp5`로 감지만 되고 파싱은 `Hwp5NotImplemented`를 반환한다.

## 라이선스

Apache-2.0. 업스트림 DVC 저작권 귀속은 `NOTICE` 참고.
