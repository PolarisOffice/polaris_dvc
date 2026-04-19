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

## 문서

- [docs/golden-tests.md](docs/golden-tests.md) — DVC parity 회귀 테스트 운영 방법
- [docs/jid-registry.md](docs/jid-registry.md) — JID 전체 레지스트리 재생성 및 엔진 확장 가이드
- [docs/dvc-parity-status.md](docs/dvc-parity-status.md) — CI에서 업스트림 DVC 빌드 시도 기록
- [docs/windows-parity-howto.md](docs/windows-parity-howto.md) — Windows PC에서 DVC.exe 실행해 expected.json 생성·검증
- [docs/parity-roadmap.md](docs/parity-roadmap.md) — 기능 parity 우선순위와 남은 작업

## 원격 푸시

GitHub Personal Access Token을 `.env.local`에 넣고 헬퍼 스크립트로 푸시한다.

```sh
cp .env.local.example .env.local
# .env.local 을 열어 GITHUB_PAT 에 값 입력 (repo 스코프 또는
# Contents: Read and write 권한)

./scripts/push.sh              # 현재 브랜치 푸시
./scripts/push.sh -u main      # 최초에 upstream 지정
```

스크립트는 PAT를 `git credential.helper`의 stdout으로 한 번만 흘려보내고,
`.git/config`·쉘 히스토리·프로세스 argv 어디에도 기록하지 않는다.
`.env.local` 자체는 `.gitignore` 처리되어 절대 커밋되지 않는다.

## 라이선스

Apache-2.0. 업스트림 DVC 저작권 귀속은 `NOTICE` 참고.
