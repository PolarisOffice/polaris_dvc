/*
 * Copyright 2022 Hancom Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#pragma once

#include "OWPML\stdafx.h"
#include "OWPML\OWPML.h"
#include "OWPML\OwpmlForDec.h"
#include "OWPMLApi\OWPMLApi.h"
#include "OWPMLUtil\HncCtrlChDef.h"

#include "DVCInterface.h"

namespace OWPMLReaderModule {

    typedef struct TabPr {
        UINT        tabPrID;
        UINT        autoTabLeft;
        UINT        autoTabRight;
    };

    enum SpacingType {
        PERCENT = 0,
    };

    typedef struct LineSpacing {
        LineSpacingType        type;            // 글자에따라(퍼센트), 고정값(pt), 여백만지정(pt), 최소(pt)
        UINT                value;            // 값
        UINT                unit;            // 단위        UVT_CHAR = 0,    UVT_HWPUNIT = 1
    };

    typedef struct Margin {                    // <hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar"> 에 해당하는 정보만 담자. 추가해야 하는 부분이 있으면 추후 구현.
        INT            indent;                    // 들여쓰기/내어쓰기
        INT            left;                    // 여백/왼쪽
        INT            right;                    // 여백/오른쪽
        INT            prev;                    // 간격/문단 위
        INT            next;                    // 간격/문단 아래
        LineSpacing    lineSpacing;
    };

    typedef struct ParaPr {
        UINT        paraPrID;
        UINT        tabPrIDRef;
        UINT        condense;                // 최소 공백
        bool        fontLineHeight;            // 글꼴에 어울리는 줄높이
        bool        snapToGrid;                // 편집 용지의 줄격자 사용
        bool        suppressLineNumbers;    // 줄번호 건너뜀 사용 
        bool        checked;                // 선택 글머리표 여부

        HAlignType    hAlineType;
        VAlignType    vAlineType;

        ParaType    headingType;            // "없음;개요 문단;번호 문단;글머리표 문단"
        UINT        headingIdRef;
        UINT        headingLevel;

        LineBreakLatinWord        breakLatinWord;
        LineBreakNonLatinWord    breakNonLatinWord;

        bool        widowOrphan;            // 외톨이줄 보호 여부
        bool        keepWithNext;            // 다음 문단과 함께 여부
        bool        keepLines;                // 문단 보호 여부
        bool        pageBreakBefore;        // 문단 앞에서 항상 쪽 나눔 여부
        bool        lineWrap;                // 한 줄로 입력 사용 시의 형식

        bool        autoSpacingEng;            // 한글과 영어 간격을 자동 조절
        bool        autoSpacingNum;            // 한글과 숫자 간격을 자동 조절

        Margin        margin;

        LineSpacing lineSpacing;

        UINT        borderFillIDRef;
        INT            borderOffsetLeft;
        INT            borderOffsetRight;
        INT            borderOffsetTop;
        INT            borderOffsetBottom;
        bool        connect;
        bool        ignoreMargin;
    };

    class RParaShape
    {
    public:
        RParaShape(OWPML::COwpmlDocumnet* document);
        ~RParaShape();

    public:
        std::vector<ParaPr*>    getParaProperties();
        std::vector<TabPr*>        getTabProperties();

        ParaPr*                    getParaPropertyForID(UINT paraID);

    private:

        bool readParaProperties(OWPML::COwpmlDocumnet* document);
        bool readTabProperties(OWPML::COwpmlDocumnet* document);

        //OWPML::CObject* FindObject(OWPML::CObject* object, UINT refID);

        std::vector<ParaPr*>    m_paraProperties;
        std::vector<TabPr*>        m_tabProperties;
    };

}