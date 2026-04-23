# polaris_dvc 기여 가이드

관심 가져주셔서 감사합니다. 이 프로젝트는 [한컴 DVC (`hancom-io/dvc`)](https://github.com/hancom-io/dvc) 를 순수 Rust 로 재구현한 것으로, **출력 바이트 수준의 호환성**을 목표로 하고 있습니다. 동작의 기준을 한컴 DVC 에 두기 때문에 일반적인 Rust 프로젝트보다 기여 흐름이 조금 더 구조화되어 있어요. PR 을 여시기 전에 이 문서를 한 번 훑어봐 주시면 좋습니다.

AI 에이전트를 활용한 작업 노트는 [`CLAUDE.md`](CLAUDE.md) 를 참고하세요. 이 문서는 사람 기여자를 위한 것입니다.

## 시작하기 전에

- **사소하지 않은 변경은 먼저 이슈로**. 버그 리포트, 기능 제안, "Parity 갭 발견" 같은 내용은 코드를 올리시기 전에 이슈를 먼저 열어주세요. 이렇게 하면 바꾸려는 동작이 혹시 의도된 한컴 DVC 호환 결정이었는지 미리 확인할 수 있습니다.
- **`third_party/dvc-upstream/` 은 수정 금지**. 참조와 라이선스 준수 목적으로 보관하는 읽기 전용 스냅샷입니다. 자세한 내용은 [`third_party/dvc-upstream/PROVENANCE.md`](third_party/dvc-upstream/PROVENANCE.md).
- **동작의 기준은 한컴 DVC**. 우리 엔진이 `Source/Checker.cpp` 와 다르게 동작하면 우리 엔진이 틀린 것입니다 — 우리 쪽 동작이 "더 깔끔해 보여도" 마찬가지예요. Parity 가 목적입니다.

## 개발 환경 설정

Rust 1.82 이상이 필요합니다 (`rust-toolchain.toml` 에 고정).

```sh
git clone https://github.com/PolarisOffice/polaris_dvc
cd polaris_dvc
cargo build --workspace
cargo test  --workspace --exclude polaris-dvc-wasm
```

WASM 타깃은:

```sh
# 한 번만 설치:
cargo install wasm-pack

wasm-pack build crates/polaris-dvc-wasm --target web
```

## PR 이 통과해야 하는 검사

CI 는 Ubuntu + macOS 에서 아래 명령을 실행합니다. Push 전에 로컬에서 먼저 돌려보시면 왕복 횟수를 줄일 수 있어요:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --exclude polaris-dvc-wasm --all-targets -- -D warnings
cargo test  --workspace --exclude polaris-dvc-wasm
wasm-pack build crates/polaris-dvc-wasm --target web
```

clippy 는 엄격 모드 (`-D warnings`) 로 돕니다. Lint 를 억제해야 할 경우 좁은 범위의 `#[allow(...)]` 를 붙이시고 한 줄 주석으로 이유를 설명해주세요.

## Golden 테스트

`testdata/golden/<nn>_...` 디렉토리들이 엔진 동작의 기준점입니다. 각 케이스는 `(doc.hwpx, spec.json, expected.json)` 의 3요소 세트 (Phase 8 부터 `expected.xml` 까지 4개) 입니다. `doc.hwpx` 바이트는 **Rust 픽스처 템플릿** (`crates/polaris-dvc-core/tests/support/mod.rs`) 에서 재현 가능하도록 설계되어 있고, `cargo test` 가 zip 을 새로 빌드해서 커밋된 바이트와 정확히 같은지 검증합니다.

픽스처 템플릿이나 엔진 체커를 수정하셨다면 재생성이 필요합니다:

```sh
POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden
```

**커밋 전에 diff 를 꼭 확인하세요.** 재생성된 `expected.json` 은 숨겨진 동작 회귀를 잡아내는 수단입니다 — 각 줄의 변경을 설명할 수 없다면 엔진 수정이 잘못되었을 가능성이 높아요.

새 golden 케이스를 추가하시려면 [`docs/golden-tests.md`](docs/golden-tests.md) 에 전체 절차가 있습니다. 요약하면:

1. `crates/polaris-dvc-core/tests/golden.rs` 에 `Case { name: "NN_descr", build, spec }` 항목 추가.
2. `mkdir testdata/golden/NN_descr`.
3. `POLARIS_REGEN_FIXTURES=1 cargo test -p polaris-dvc-core --test golden` 실행.
4. 생성된 파일 4개 + 소스 수정 같이 커밋.

## 새 JID (에러 코드) 추가

[`CLAUDE.md`](CLAUDE.md) 의 "How to wire a new JID into the engine" 섹션 템플릿을 따라주세요. 요약:

1. `crates/polaris-dvc-core/src/jid_registry.rs` 에서 상수 찾기 (한컴 DVC 에서 자동 생성되므로 직접 수정하지 말 것).
2. `src/error_codes.rs` 의 `jid` 모듈에 짧은 이름의 alias 추가.
3. `ErrorCode::text(self)` 에 사람이 읽을 수 있는 메시지 arm 추가 (가능하면 한컴 DVC 의 표현 그대로).
4. 규칙이 새 JSON 필드를 도입한다면 `rules/schema.rs` 확장.
5. `engine.rs` 에 `check_<thing>()` 함수 추가. 반드시 `ctx.push(v)` 경유 — `ctx.records` 에 직접 push 금지.
6. 새 코드 경로를 검증하는 golden 케이스 추가.

## 커밋 · PR 컨벤션

- **커밋 메시지**: 명령형, 제목 72자 이내, 본문에는 *왜* 바꾸는지 설명. 스타일 참고는 최근 `git log` 를 봐주세요.
- **한 PR 한 주제**. 리팩토링과 기능 변경은 섞지 말아주세요 — 리뷰어가 바이트 diff 를 어느 의도에서 나온 건지 판단하기 어려워집니다.
- **PR 이 열려 있을 때 amend 나 force-push 금지**. 새 커밋을 쌓아주세요. GitHub 측에서 squash-merge 로 정리됩니다. 리포 전반에 적용되는 규칙이며 [`CLAUDE.md`](CLAUDE.md) 에도 명시되어 있습니다.
- **Sign-off 는 선택 사항**. Apache-2.0 프로젝트라 CLA 는 없습니다.

## 한컴 DVC 와의 Parity

`--dvc-strict` 프로파일에서 한컴 DVC 의 `DVC.exe` 와 **바이트 수준 호환**을 목표로 합니다. [`docs/parity-roadmap.md`](docs/parity-roadmap.md) 에 JID 별 커버리지가 정리되어 있고, [`docs/windows-parity-howto.md`](docs/windows-parity-howto.md) 는 Windows 에서 `DVC.exe` 를 돌려 `expected.json` 을 새로 만드는 절차를 안내합니다. 기여에 Windows 환경이 필수는 아니에요 — 리포에 들어 있는 golden 테스트는 Linux/macOS 에서도 돕니다. 다만 Windows 환경이 있으시고 parity 갭을 메우는 데 도움을 주실 수 있다면 그 문서를 참고해주세요.

## 행동 강령

[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) 을 따릅니다. 위반 신고는 [`SECURITY.md`](SECURITY.md) 에 있는 채널을 통해 비공개로 접수됩니다.

## 라이선스

기여해주시는 내용은 Apache License 2.0 으로 라이선스됨에 동의하시는 것으로 간주합니다 (이 프로젝트 및 한컴 DVC 와 동일). 자세한 내용은 [`LICENSE`](LICENSE), [`NOTICE`](NOTICE) 참고.
