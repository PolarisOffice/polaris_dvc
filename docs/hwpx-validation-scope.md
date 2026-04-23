# HWPX 검증 범위 — polaris 의 정체성 재정의

## 원래 스코프 (1.0 전)

한컴 DVC (hancom-io/dvc) 의 Rust 포팅. 사용자가 준 **rule spec JSON**
(charshape·parashape·table 등의 카테고리별 허용 규칙) 대로 HWPX 문서가
지켜지는지 검증. 결과를 upstream DVC.exe 와 byte-exact parity 로 맞춤.

## 확장된 스코프 (1.x 이후)

**HWPX 종합 검증 툴**. 단일 rule spec 을 넘어서 문서 자체의 무결성·표준
적합성까지 검사. 네 축:

```
입력: HWPX 바이트
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ Axis 1  — Rule conformance (upstream DVC 호환)     │
│   사용자 spec 의 허용 규칙을 따르는가              │
│   JID 1000-7999  (CharShape/ParaShape/Table/...)   │
│                                                     │
│ Axis 2  — Structural integrity                      │
│   Cross-ref / ZIP / manifest / lineseg 정합성      │
│   JID 11000-11999                                   │
│                                                     │
│ Axis 3  — Container well-formedness                 │
│   ZIP 형식 자체, 필수 entry, 금지 extras           │
│   JID 12000-12999                                   │
│                                                     │
│ Axis 4  — Schema conformance (KS X 6101)           │
│   OWPML XSD 대비 각 XML 적합성                     │
│   JID 13000-13999                                   │
│                                                     │
│ Axis 5  — Structural invariants (선택, Phase 4)    │
│   Page number monotonic, table span 합 등          │
│   JID 14000-14999                                   │
│                                                     │
│ Axis 6+ — Encoding / Semantic warnings (선택)      │
│   JID 15000+ / 16000+                               │
└─────────────────────────────────────────────────────┘
         │
         ▼
ViolationRecord 리스트 (1-6 의 union, severity 포함)
```

## 왜 확장하나

1. **DVC 호환만으로는 부족** — upstream DVC 는 Windows 전용 바이너리이고,
   그 의존 라이브러리 (`hwpx-owpml-model`) 는 Hancom Docs 에서 생성된
   HWPX 파일의 일부 refID 를 구현하지 않아 실제 범용 검증 툴로는 한계가
   있다 (자세한 내용: `docs/dvc-parity-handoff.md`). DVC 호환은 여전히
   가치 있지만, 공개 레퍼런스 체커로는 그 위에 표준 기반 검증을 쌓아야 함
2. **공개 도메인의 빈 자리** — OWPML XSD 로 HWPX 를 검증하는 공개 툴이
   없다. 한컴 `hwpx-owpml-model` 은 로딩만 하고 XSD 검증 안 함.
   `polaris-dvc` 가 KS X 6101 레퍼런스 체커 자리를 채움
3. **기존 구조 재사용 용이** — JID 레지스트리·엔진·순수-Rust HWPX 파서·
   출력 모델 이미 준비됨. 새 카테고리 블록 몇 개 추가 + check 함수
   추가로 수평 확장
4. **LLM 시대 검증 수요** — AI 로 HWPX 를 생성하는 사례 증가. 생성물의
   구조·스키마 적합성을 자동 판정하는 도구 필요

## 포지셔닝

```
프로젝트 이름:  polaris_dvc   (유지)
패키지:        polaris-dvc-*  (유지)
태그라인 변경: "HWPX Document Validation Checker (DVC)"  →
              "HWPX 종합 검증 툴 — 규칙·구조·스키마·무결성 종합 체커"

DVC 호환은 **기능 중 하나**. Spec 모드 (KS X 6101 표준 기준 검증) 가
장기 축. 공식 reference 체커가 되는 것이 목표.
```

## Axis 별 JID 블록 할당

| 블록 | 카테고리 | 상태 |
|---|---|---|
| 1000-7999 | Rule conformance (DVC 호환) | ✅ 217 개, 레지스트리 완성 |
| 10000-10999 | Extended (polaris 독자 확장) | ✅ 일부 사용 중 |
| **11000-11999** | Integrity | 🚧 Phase 1 에서 10개 → 50+ 개 확장 |
| **12000-12999** | Container | 🚧 Phase 2 신규 |
| **13000-13999** | Schema conformance | 🚧 Phase 3 신규 |
| 14000-14999 | Invariant (예약) | 📋 Phase 4 |
| 15000-15999 | Encoding (예약) | 📋 Phase 5 |
| 16000-16999 | Semantic warning (예약) | 📋 Phase 5 |

## 사용자 관점 흐름

### 1. 기본 (모든 Axis)

```sh
polaris-dvc -t rules.json doc.hwpx
# → container / integrity / schema / rule 모두 검사
# → Violation 리스트 (카테고리별 분류된 severity 포함)
```

### 2. Rule 검증만 (DVC 호환 모드)

```sh
polaris-dvc --dvc-strict -t rules.json doc.hwpx
# → 1000-7999 범위만, upstream DVC.exe 가 구현한 JID 만
```

### 3. Schema 검증만

```sh
polaris-dvc --only=schema doc.hwpx
# → rule spec 없이도 OK. 순수 표준 적합성만
```

### 4. Container 검증만 (빠른 sanity check)

```sh
polaris-dvc --only=container doc.hwpx
# → ZIP 수준만, XML 파싱 안 함. 대량 파일 스크리닝용
```

## Crate 분리

```
crates/
├── polaris-dvc-core/     엔진·에러코드·출력·Integrity/Container 체크   (기존)
├── polaris-dvc-hwpx/     HWPX 파서                                       (기존)
├── polaris-dvc-format/   포맷 감지 + dispatch trait                       (기존)
├── polaris-dvc-schema/   신규: OWPML 스키마 모델 + validator             (Phase 3)
├── polaris-dvc-cli/      CLI                                              (기존)
└── polaris-dvc-wasm/     WASM 래퍼                                        (기존)

tools/
├── gen-jids/      upstream JID 헤더 → jid_registry.rs                (기존)
└── gen-owpml/     표준 XSD → generated_owpml.rs                      (Phase 3)
```

## 개발 로드맵 개요

- **Phase 0** (준비) — .gitignore, standards/README, 본 문서, README 태그라인 재작성
- **Phase 1** — Integrity 확장 (cross-ref 검사 대폭 추가)
- **Phase 2** — Container JID 12000 블록 신설
- **Phase 3** — `polaris-dvc-schema` 크레이트 + codegen + XSD 검증 (실용적 80% 커버)
- **Phase 4** — Invariant 체크 (예약)
- **Phase 5** — Encoding / Semantic warning (예약)
- **Phase 6** — severity 필터링 + web UI 대시보드 (예약)

세부 설계는 해당 Phase 작업 시 PR 에 커밋 단위로.
