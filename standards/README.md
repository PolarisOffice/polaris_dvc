# `standards/` — OWPML 표준 자료 (로컬 전용)

이 디렉토리는 **KS X 6101** (한국산업표준: HWPX 문서 형식 명세) 원본 자료를
담기 위한 자리다. 표준 문서 자체는 **KATS / TTA 의 저작권**이며 재배포가
제한되므로 `.gitignore` 로 제외되어 있다 — 오직 이 `README.md` 만 커밋된다.

## 무엇이 들어가야 하나

```
standards/
├── README.md                 (이 파일, 커밋 대상)
├── KSX6101_KR.pdf            KS X 6101 표준 본문 (한국어판 PDF)
└── KSX6101_OWPML/
    ├── HWPMLBodySchema.xsd
    ├── HWPMLCoreSchema.xsd
    ├── HWPMLHeaderSchema.xsd
    ├── HWPMLHistorySchema.xsd
    ├── HWPMLMasterPageSchema.xsd
    ├── HWPMLParaListSchema.xsd
    └── HWPMLVersionSchema.xsd
```

## 어디서 받는가

- **KATS e-나라 표준인증** (<https://standard.go.kr>) 에서
  "KS X 6101" 로 검색 → PDF 다운로드
- 부속 XSD 는 한컴 개발자 사이트에 공개 또는 TTA 가 별도 배포한 버전 사용

두 자료 모두 **원저작권은 KATS/TTA 에 있으며 본 레포에 재배포 금지**.

## 왜 필요한가 — 그리고 언제 필요한가

`crates/polaris-dvc-schema/` 의 XSD-파생 Rust 코드 (`generated_owpml.rs`)
는 `tools/gen-owpml/` 이 이 디렉토리를 읽어 생성한 **파생 팩트 데이터**다.
Element 이름·자식 관계·속성 타입 등 **사실적 내용만** 추출하며 원본의
documentation 텍스트·다이어그램은 복사하지 않는다.

일반 개발 / 빌드에는 이 디렉토리가 **필요하지 않다**. 커밋된
`generated_owpml.rs` 만으로 완결된다. 표준 개정 (예: 2027 판) 이 나와
재생성해야 할 때만 이 자료가 있어야 한다:

```sh
# 로컬에 standards/ 배치 후:
cargo run --manifest-path tools/gen-owpml/Cargo.toml
# -> crates/polaris-dvc-schema/src/generated_owpml.rs 갱신
```

## 저작권 / 배포 경계

- ✅ 원본 XSD 구조로부터 추출한 **팩트 데이터** (element 이름, 자식 허용 관계,
  속성 required 여부, enum 값 리스트) — Rust 소스로 커밋 OK. Feist v.
  Rural 에 준해 순수 사실은 저작권 대상이 아님
- ❌ 원본 XSD 의 `<xs:annotation><xs:documentation>` 한국어 설명, 표준 본문의
  산문·다이어그램·예시 코드 — 재배포 금지. `gen-owpml/` 는 documentation
  노드 버림
- ❌ PDF, XSD 파일 **자체** — 이 디렉토리 밖으로 복사·커밋·업로드 금지

## 유사 선례

`third_party/dvc-upstream/` 는 Apache-2.0 로 공개된 한컴 upstream 을 vendored
하므로 재배포 OK. `standards/` 는 정반대 — 참조 전용, 재배포 불가. 카테고리
자체가 다르다.
