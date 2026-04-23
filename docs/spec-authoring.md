# Spec 파일 작성 가이드

`polaris-dvc` 에 `-t` 로 넘기는 **규칙 spec 파일**은 문서가 지켜야 할
폰트 · 크기 · 스타일 · 표 속성 등을 JSON 으로 선언한 파일이다. 이
문서는 spec 파일 하나를 처음 만들 때 필요한 것만 모아 설명한다.

## 스펙 파일이란

> 한컴 DVC 와 동일한 스키마를 따르는 검증 규칙의 집합.
>
> `schemas/jsonFullSpec.json` 은 **"사용할 수 있는 필드의 목록과
> 타입"** 을 기술한 레퍼런스다. 실제 검증에는 그 목록에서 필요한
> 필드만 골라 **구체적인 값**으로 채운 파일을 쓴다.

예시로 비교해보면 이렇다:

```jsonc
// schemas/jsonFullSpec.json 의 일부 — "이 필드는 number 타입이다" 를 설명
{
  "charshape": {
    "fontsize": { "min": { "type": "number" }, "max": { "type": "number" } }
  }
}
```

```jsonc
// 실제 spec.json — "글자 크기는 10pt 이상 12pt 이하여야 한다" 를 선언
{
  "charshape": {
    "fontsize": { "min": 10, "max": 12 }
  }
}
```

아래쪽 "실제 spec" 파일을 `-t` 에 넘기면 된다.

## 가장 단순한 시작

규칙 없이 empty object 를 넘기면 규칙 검사는 건너뛰고 구조·컨테이너 축만
돈다 (HWPX 파일 자체가 온전한지 확인).

```json
{}
```

`polaris-dvc -t minimal.json your.hwpx` 처럼 쓴다.

## 카테고리별 빠른 치트시트

### charshape — 글자 모양

```json
{
  "charshape": {
    "font": ["바탕", "돋움", "굴림"],
    "fontsize": { "min": 10, "max": 12 },
    "bold": false,
    "italic": false
  }
}
```

주요 필드:

| 필드 | 타입 | 설명 |
|---|---|---|
| `font` | string \| string[] | 허용 폰트 이름 (리스트면 그 안에서만 허용) |
| `fontsize` | number \| `{min, max}` | 글자 크기. 단일 값 또는 범위 |
| `bold` / `italic` / `underline` / `strikeout` | boolean | 해당 속성이 이 값이어야 함 |
| `ratio` | number \| `{min, max}` | 장평 (50–150) |
| `spacing` | number \| `{min, max}` | 자간 (0–100) |

### parashape — 문단 모양

```json
{
  "parashape": {
    "linespacingvalue": 160,
    "indent": 0,
    "alignment": "JUSTIFY"
  }
}
```

주요 필드:

| 필드 | 타입 | 설명 |
|---|---|---|
| `alignment` | enum | `JUSTIFY`, `LEFT`, `RIGHT`, `CENTER`, `DISTRIBUTE` |
| `linespacing` | enum | `PERCENT`, `FIXED`, `BETWEEN_LINES`, `MINIMUM` |
| `linespacingvalue` | number | 줄 간격 값 (PERCENT면 %) |
| `indent` | number | 들여쓰기 HWPUNIT |

### table — 표

```json
{
  "table": {
    "border": [
      { "position": 1, "bordertype": 1 }
    ],
    "bgfill-type": 1,
    "table-in-table": false
  }
}
```

주요 필드:

| 필드 | 타입 | 설명 |
|---|---|---|
| `border` | array of objects | `position` (외곽 1, 내부 2 등), `bordertype` (선 종류), `size`, `color` |
| `bgfill-type` | enum | `0`=없음, `1`=단색, `2`=패턴, `3`=그라데이션, `4`=이미지 |
| `table-in-table` | boolean | 표 안의 표를 허용할지 |

### style — 스타일 사용 제한

```json
{ "style": { "permission": false } }
```

`permission: false` 면 문서 내 스타일 참조 자체를 금지.

### hyperlink / macro — 보안 관련

```json
{
  "hyperlink": { "permission": false },
  "macro": { "permission": false }
}
```

하이퍼링크 / 매크로 포함을 금지한다. 공공·정부 문서 검증에서 자주 쓴다.

### specialcharacter — 특수문자 범위

```json
{
  "specialcharacter": { "minimum": 32, "maximum": 1048575 }
}
```

허용할 유니코드 코드포인트 범위.

## 종합 예제

카테고리별 규칙을 조합한 실전 spec:

```json
{
  "charshape": {
    "font": ["바탕", "돋움", "굴림"],
    "fontsize": { "min": 10, "max": 12 },
    "bold": false,
    "ratio": 100
  },
  "parashape": {
    "linespacingvalue": 160,
    "indent": 0
  },
  "table": {
    "border": [{ "position": 1, "bordertype": 1 }],
    "table-in-table": false
  },
  "style": { "permission": false },
  "hyperlink": { "permission": false },
  "macro": { "permission": false },
  "specialcharacter": { "minimum": 32, "maximum": 1048575 }
}
```

## 실제 샘플

리포의 `testdata/golden/*/spec.json` 은 각각 특정 규칙 하나를 위반시키도록
설계된 실전 spec 예제다. 카테고리별로 최소 한 개씩 들어 있으니 복붙
시작점으로 쓰기 좋다:

- `testdata/golden/01_clean/spec.json` — 완전한 all-categories 예제
- `testdata/golden/02_fontsize_mismatch/spec.json` — charshape.fontsize 단일 값
- `testdata/golden/11_table_border_type_mismatch/spec.json` — table.border
- `testdata/golden/14_parashape_indent_mismatch/spec.json` — parashape.indent
- `testdata/golden/24_table_bgfill_type_mismatch/spec.json` — table.bgfill

## 전체 필드 목록

`schemas/jsonFullSpec.json` 에 모든 카테고리의 허용 필드가 타입 정보와
함께 나열돼 있다. 작성 중 필드 이름이나 enum 값이 생각 안 날 때
참조하면 된다 (이 파일 자체를 `-t` 에 넘기지는 말 것 — 위의 설명 참조).

## 자주 하는 실수

- **`fontsize: 10`** 과 **`fontsize: { "min": 10, "max": 10 }`** 는
  같은 효과지만, `{ "min": X, "max": X }` 쪽이 명시적이라 권장.
- **enum 값의 대소문자**: `alignment` 는 `"JUSTIFY"` 가 맞고
  `"justify"` 는 매치되지 않는다. jsonFullSpec 의 표기를 그대로 따를 것.
- **`font` 문자열 vs 배열**: 단일 폰트만 허용하려면 `"font": "바탕"`,
  여러 폰트 허용하려면 `"font": ["바탕", "돋움"]`. 빈 배열은
  "어떤 폰트도 허용 안 함" 이 되므로 주의.
- **한컴 DVC 호환 모드 (`--dvc-strict`) 에서 무시되는 필드가 있음**:
  표의 margin·bgfill·caption 계열 일부 필드는 한컴 DVC 본체가
  no-op 로 처리하도록 구현돼 있다. polaris 의 Extended 모드에서는
  검증되지만 `--dvc-strict` 를 켜면 결과에서 빠진다.
