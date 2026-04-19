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

    class CCharShape : public ICharShape
    {
    public:
        CCharShape(std::string dvcFile);
        CCharShape(LPCSTR document);
        ~CCharShape();

    public:        
        DVCType getType() { return DVCType::CharShape; }
        std::map<std::string, int>    getMapData() { return m_mapCheckedList; }
        std::vector<UINT> getCheckedList() { return m_checkedList; }

        JsonUINT        getFontSize();
        LangType        getLangType(); 
        std::vector<std::string> getFontNames();
        JsonUINT        getRSize();        // 10 ~ 250
        JsonINT            getPosition();    // -100 ~ 100
        JsonUINT        getRatio();        // 50 ~ 200
        JsonINT            getSpacing();    // -50 ~ 50

        bool            getBold();
        bool            getItalic();
        bool            getUnderline();
        bool            getStrikeout();
        bool            getOoutline();
        bool            getEmboss();
        bool            getEngrave();
        bool            getShadow();
        bool            getSupscript();
        bool            getSubscript();

        ShadowType        getShadowType();
        INT                getShadowXDirection();    // -100 ~ 100
        INT                getShadowYDirection();    // -100 ~ 100
        UINT            getShadowColor();

        LinePosition    getUnderlinePos();
        LineShape        getUnderlineShape();
        UINT            getUnderlineColor();

        LineShape        getStrikeoutShape();
        UINT            getStrikeColor();

        LineShape        getOutlineShape();
        bool            getEmptyspace();
        UINT            getPoint();
        bool            getKerning();

        BorderInfo*        getBorderInfo(LinePosition pos);
        std::vector<BorderInfo*> getBorderInfos();
        UINT            getBackgroundColor();
        UINT            getPattonColor();
        PattonType        getPattonType();


    private:
        bool parsingDVCFile(std::string dvcFile);
        bool parsingDVCDocument(LPCSTR document);

        bool parsingElement(Json::Value root, void* Obj = 0);

        std::string                    m_dvcFile;
        std::vector<UINT>            m_checkedList;    // json 으로 설정되어 있는 항목들을 포함하고 있는 벡터. 이 값을 보고 실제 어떤 값들이 설정되어 있는지 확인한다.
        std::map<std::string, int>    m_mapCheckedList;

        JsonUINT                        m_fontSize;
        std::vector<LangSet*>        m_langSet;

        LangType                    m_langType;
        std::vector<std::string>    m_fontName;
        JsonUINT                    m_rSize;
        JsonINT                        m_position;
        JsonUINT                    m_ratio;
        JsonINT                        m_spacing;

        bool                        m_bBold;
        bool                        m_bItalic;
        bool                        m_bUnderline;
        bool                        m_bStrikeout;
        bool                        m_bOoutline;
        bool                        m_bEmboss;
        bool                        m_bEngrave;
        bool                        m_bShadow;
        bool                        m_bSupscript;
        bool                        m_bSubscript;
                                    
        ShadowType                    m_shadowType;
        JsonUINT                    m_shadowXDirection;
        JsonUINT                    m_shadowYDirection;
        UINT                        m_shadowColor;
                                    
        LinePosition                m_underlinePos;
        LineShape                    m_underlineShape;
        UINT                        m_underlineColor;
                                    
        LineShape                    m_strikeoutShape;
        UINT                        m_strikeColor;
                                    
        LineShape                    m_outlineShape;
        bool                        m_emptyspace;
        UINT                        m_point;        // 강조점
        bool                        m_kerning;

        std::vector<BorderInfo*>    m_borderInfo;
        UINT                        m_backgroundColor;
        UINT                        m_pattonColor;
        PattonType                    m_pattonType;
    };

}