# CLI 호환성 — polaris-dvc vs upstream DVC

polaris 의 CLI (`polaris-dvc`) 는 업스트림 [hancom-io/dvc](https://github.com/hancom-io/dvc) 의
`ExampleWindows.exe` 플래그 표면을 참조해 만들었다. 이 문서는 **실제 동작 기준** 으로 둘 사이의
일치점과 의도적으로 다른 부분을 정리한다. 업스트림 플래그 정의의 실제 소스는
`CommandParser.cpp::parsingShortOption` / `parsingLongOption` 이며, JSON 키 이름은
`Source/JsonModel.h` 의 `JIN_*` 매크로.

## 업스트림 전체 플래그 (`ExampleWindows.exe`)

업스트림 README 는 아래 테이블을 싣고 있지만, 일부 설명이 실제 소스와 맞지 않는다. 실제
동작을 기준으로 정리했다.

| 짧은 | 긴 | 분류 | 동작 |
|---|---|---|---|
| `-j` | `--format=json` | 출력 형식 | JSON 출력 (기본) |
| `-x` | `--format=xml` | 출력 형식 | XML 출력 (업스트림 미구현, 호출 시 `"Option not yet available"` 에러 반환) |
| `-c` | `--console` | 결과 출력 | 콘솔 출력 (기본) |
|  | `--file=[PATH]` | 결과 출력 | 지정 파일에 출력 저장 |
| `-s` | `--simple` | 체크 수준 | 첫 오류 발견 즉시 중단 |
| `-a` | `--all` | 체크 수준 | 모든 항목 검사 (기본) |
| `-d` | `--default` | 출력 옵션 | Default 필드 세트 (기본) |
| `-o` | `--alloption` | 출력 옵션 | 모든 조건부 필드 포함 |
| `-t` | `--table` | 출력 옵션 | 테이블 관련 필드만 (cell 단위 X) |
| `-i` | `--tabledetail` | 출력 옵션 | 테이블 셀 단위 |
| `-p` | `--shape` | 출력 옵션 | 도형 관련 필드 |
| `-y` | `--style` | 출력 옵션 | 스타일 관련 필드 |
| `-k` | `--hyperlink` | 출력 옵션 | 하이퍼링크 관련 필드 |
| `-h` / `-H` | `--help` | 공통 | 도움말 |
| `-v` | `--version` | 공통 | 버전 |

**스펙 파일 / HWPX 파일은 positional args**. 업스트림 파서는 `-` 로 시작하지 않는
첫 번째 인자를 `m_dvcFilepath` (스펙), 두 번째를 `m_targetFilepath` (HWPX) 로 저장
한다. 순서만 맞으면 어떤 플래그와도 섞어 쓸 수 있다.

업스트림 README Demo 의 다음 예제:

```
ExampleWindows.exe -j --file=Result.json -s -t test.json "005_busan.hwpx"
```

여기서 `-t` 는 **OutputOption::Table 토글 플래그**이고, `test.json` 과 `005_busan.hwpx`
는 뒤따르는 positional args 다. `-t` 가 "스펙 파일을 가리키는 플래그" 처럼 보이지만
실제로는 그렇지 않다.

## polaris-dvc 의 매핑

| polaris | 업스트림 대응 | 메모 |
|---|---|---|
| `-j` | `-j` | 동일 — JSON 출력 |
| `-x` | `-x` | 동일 — 단, **polaris 는 Extended 프로파일에서 실제 XML 출력**. `--dvc-strict` 하에서만 업스트림처럼 exit 2 로 NotYet 리턴 |
| `--format=json\|xml` | `--format=json\|xml` | 동일 |
| `--file=PATH` | `--file=PATH` | 동일 |
| `-s` / `--simple` | `-s` / `--simple` | 동일 — 첫 오류에서 중단 |
| `-a` / `--all` | `-a` / `--all` | 동일 (기본값) |
| **`-t SPEC`** | 다름 — **polaris 는 스펙 파일 경로**, 업스트림 은 OutputOption::Table 토글 | 의도적 divergence. 업스트림 Demo 의 혼란스러운 `-t <spec>` 사용 패턴과 시각적으로 일치시켜 실수 최소화 |
| `--output-option=<set>` | `-d`/`-o`/`-t`/`-i`/`-p`/`-y`/`-k` 단일 플래그들 | 업스트림의 7 개 단일-문자 토글 대신 하나의 `--output-option` 으로 통합 (`default`, `all`, `table`, `table-detail`, `shape`, `style`, `hyperlink`). `-t` 단일문자가 스펙 경로로 재정의됐기 때문에 선택의 여지 없었다 |
| `--dvc-strict` | 없음 | polaris 전용 — 업스트림이 실제 구현한 JID 만 출력 (Extended 프로파일에서는 업스트림이 no-op 처리한 JID 도 검사한다) |
| `-` (positional) | `-` | 동일 — stdin 에서 HWPX 바이트 읽기 |

## 스펙 파일 vs 스키마 파일 — 혼동 주의

업스트림의 `sample/jsonFullSpec.json` (우리는 `schemas/jsonFullSpec.json` 으로 vendored) 은
**JSON Schema 스타일의 참조 문서** — `"fontsize": { "type": "number" }` 처럼 각 필드의
허용 타입·범위·enum 을 메타-기술. `[Json schema]` prelude 로 시작한다.

**실제 validation 실행에는 이 파일을 그대로 쓸 수 없다.** 거기에서 원하는 필드만 뽑아
구체적 값으로 채운 스펙 파일을 따로 만들어야 한다. 업스트림 README Demo 가 쓰는
`sample/test.json` 은 그런 실제 스펙의 예시 (5 KB, 구체적 값들).

우리 리포 안의 실제 스펙 예시:
- `third_party/dvc-upstream/sample/test.json` — 업스트림 공식 예제 (vendored)
- `testdata/golden/01_clean/spec.json` — 가장 단순한 우리 fixture (4 줄)
- `testdata/golden/<case>/spec.json` — 목적별 45 개 fixture

## 기본 실행 예제

### polaris

```sh
# 기본: JSON 출력, 모든 조건부 필드 포함, stdout 으로
cargo run -p polaris-dvc-cli -- \
    -t third_party/dvc-upstream/sample/test.json path/to/document.hwpx

# 파일로 저장 + 첫 오류에서 중단
cargo run -p polaris-dvc-cli -- \
    -j --file=out.json -s \
    -t my-spec.json path/to/document.hwpx

# DVC-strict: 업스트림과 바이트 동일성 목표
cargo run -p polaris-dvc-cli -- \
    -j --file=out.json --dvc-strict \
    -t my-spec.json path/to/document.hwpx

# stdin 에서 HWPX 읽기 (파이프라인 친화)
cat doc.hwpx | cargo run -p polaris-dvc-cli -- \
    -j -t my-spec.json -
```

### 업스트림 DVC

업스트림 README 의 Demo 명령:

```
ExampleWindows.exe -j --file=Result.json -s -t test.json "005_busan.hwpx"
```

여기서 `test.json` 은 **스펙 파일** (positional arg), `-t` 는 `OutputOption::Table` 토글.
polaris 에서 같은 의도를 쓰려면:

```sh
cargo run -p polaris-dvc-cli -- \
    -j --file=Result.json -s --output-option=table \
    -t test.json path/to/document.hwpx
```

(polaris 의 `-t` 는 여기서 "스펙 파일을 test.json 으로 지정" 을 의미한다.)

## Exit code 정책 (polaris)

업스트림은 exit code 를 문서화하지 않는다. polaris 는 다음과 같이 정의한다:

| Exit | 의미 |
|---|---|
| 0 | 검증 성공, 위반 0 건 |
| 1 | 위반 감지됨 |
| 2 | 사용법 오류 (플래그 오류, `--dvc-strict` 하의 `-x`) |
| 3 | 파싱 실패 (HWPX 읽기 실패, spec JSON 파싱 실패) |

## 업스트림과의 알려진 동작 차이

| 항목 | polaris | 업스트림 |
|---|---|---|
| `-x` (XML 출력) | Extended: 실제 XML, DVC-strict: NotYet | 항상 NotYet |
| 미구현 JID (margin-*, bgfill-*, caption-* 등) | Extended: 검증함, DVC-strict: 무시 | 항상 `break;` no-op |
| 구조 무결성 JID (11000-11999, polaris 전용) | Extended 에서 emit | 개념 없음 |
| 규칙 JSON 에 등록되지 않은 최상위 키 | warning 수집, 검증 계속 | 현재는 `switch (mapIter->second)` 에서 crash 가능 (업스트림 bug) |
| 규칙 값 scalar vs `{min,max}` | `Range64` 가 둘 다 수용 | `min`/`max` 없는 scalar 주면 0 으로 읽음 |

## 참고 자료

- `crates/polaris-dvc-cli/src/main.rs` — clap derive 기반 정의
- `third_party/dvc-upstream/CommandParser.cpp` — 업스트림 원본 파서 로직
- `third_party/dvc-upstream/Source/JsonModel.h` — 스펙 키 이름 매크로
- `docs/dvc-parity-handoff.md` — DVC.exe 와 바이트 parity 달성 시도 기록
