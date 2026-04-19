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
#include <string>

#include "DVCInterface.h"
#include "JsonModel.h"

namespace CheckListModule {
    class CParaShape : public IParaShape
    {
    public:
        CParaShape(std::string dvcFile);
        CParaShape(LPCSTR document);
        ~CParaShape();

    public:
        DVCType getType() { return DVCType::ParaShape; }
        std::map<std::string, int>    getMapData() { return m_mapCheckedList; }
        std::vector<UINT>        getCheckedList() { return m_checkedList; }

        HAlignType                getHorizontal();
        JsonINT                    getMarginLeft();    // -415 ~ 415
        JsonINT                    getMarginRight();    // -415 ~ 415

        bool                    getFirstline();
        JsonUINT                getIndent();        // 0 ~ 400
        JsonUINT                getOutdent();        // 0 ~ 400
        LineSpacingType            getLinespacing();
        JsonUINT                getLinespacingvalue();    // 0 ~ 500
        JsonUINT                getSpacingParaup();        // 0 ~ 841.8
        JsonUINT                getSpacingParabottom(); // 0 ~ 841.8
        bool                    getSpacingGridpaper();
        bool                    getLinebreakKorean();
        LineBreakLatinWord        getLinebreakEnglish();
        JsonUINT                getLinebreakCondense();    // 25 ~ 100

        ParaType                getParaType();
        UINT                    getParatypeValue();

        bool                    getWidowOrphan();
        bool                    getKeppWithNext();
        bool                    getKeepLinesTogether();
        bool                    getPageBreakBefore();
        bool                    getFontLineHeight();
        bool                    getLineWrap();
        bool                    getAutoSpaceEasianEng();
        bool                    getAutoSpaceEasianNum();
        VAlignType                getVerticalAlign();
        std::vector<TabTypes*>    getTabTypes(TabType type);
        bool                    getAutotabIntent();
        bool                    getAutotabPararightEnd();
        JsonUINT                getBaseTabSpace();        // 0 ~ 425

        BorderInfo*                getBorderInfo(LinePosition pos);
        UINT                    getBackgroundColor();
        UINT                    getPattonColor();
        PattonType                getPattonType();

        bool                    getSpacingLeft();
        bool                    getSpacingRight();
        bool                    getSpacingTop();
        bool                    getSpacingBottom();
        bool                    getSpacingIgnore();
                

    private:
        bool parsingDVCFile(std::string dvcFile);
        bool parsingDVCDocument(LPCSTR document);

        bool parsingElement(Json::Value root, void* Obj = 0);

        std::string                    m_dvcFile;
        std::vector<UINT>            m_checkedList;    // json 으로 설정되어 있는 항목들을 포함하고 있는 벡터. 이 값을 보고 실제 어떤 값들이 설정되어 있는지 확인한다.
        std::map<std::string, int>    m_mapCheckedList;

        HAlignType                m_horizontal;
        JsonINT                    m_marginLeft;
        JsonINT                    m_marginRight;

        bool                    m_Firstline;
        JsonUINT                m_indent;
        JsonUINT                m_outdent;
        LineSpacingType            m_linespacing;
        JsonUINT                m_linespacingvalue;
        JsonUINT                m_spacingParaup;
        JsonUINT                m_spacingParabottom;
        bool                    m_spacingGridpaper;
        bool                    m_linebreakKorean;    // false = "글자", true = "어절"
        LineBreakLatinWord        m_linebreakEnglish;    // "단어", "하이픈", "글자"
        JsonUINT                m_linebreakCondense;

        ParaType                m_paraType;
        UINT                    m_paratypeValue;

        bool                    m_widowOrphan;
        bool                    m_keepWithNext;
        bool                    m_keepLinesTogether;
        bool                    m_pageBreakBefore;
        bool                    m_fontLineHeight;
        bool                    m_lineWrap;
        bool                    m_autoSpaceEasianEng;
        bool                    m_autoSpaceEasianNum;
        VAlignType                m_verticalAlign;
        std::vector<TabTypes*>    m_tabTypes;
        //TabTypes*                m_tabType;
        bool                    m_autotabIntent;
        bool                    m_autotabPararightEnd;
        JsonUINT                m_baseTabSpace;

        std::vector<BorderInfo*>    m_borderInfo;
        UINT                        m_bgColor;
        UINT                        m_bgPattonColor;
        PattonType                    m_bgPattonType;

        bool                    m_spacingLeft;
        bool                    m_spacingRight;
        bool                    m_spacingTop;
        bool                    m_spacingBottom;
        bool                    m_spacingIgnore;
    };
}
