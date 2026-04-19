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

namespace OWPMLReaderModule {

    enum FontFaceType {
        NANGUL = 0,
        LATIN,
        HANJA,
        JAPANESE,
        OTHER,
        SYMBOL,
        USER
    };

    typedef struct CharPrInfo {
        UINT            fontRef;        // hangul의 값만 일단 저장하다. 추후 다른 언어도 추가하도록.
        UINT            ratio;
        INT                spacing;
        UINT            relSz;
        INT                offset;

        bool            bold;
        bool            italic;
        bool            underline;
        bool            strikeout;
        bool            outline;
        bool            emboss;
        bool            engrave;
        bool            shadow;
        bool            supscript;
        bool            subscript;

        UINT            shadowType;
        INT                shadowX;
        INT                shadowY;
        COLORREF        shadowColor;
    };

    typedef struct CharPr {
        UINT                    charPrID;
        UINT                    height;        // fontsize 1000 = 10pt
        COLORREF                textColor;
        COLORREF                shadeColor;
        bool                    useFontSpace;
        bool                    useKerning;
        OWPML::SYMBOLMARKTYPE    symMark;    // enum
        UINT                    borderFillIDRef;
        CharPrInfo                charPrInfo;
    };

    typedef struct FontFace {
        OWPML::FONTFACELANGTYPE        fontfaceType;
        std::map<UINT, LPCWSTR>        fonts;    // <id, face>
    };

    class RCharShape
    {
    public:
        RCharShape(OWPML::COwpmlDocumnet* document);
        ~RCharShape();

    public:
        std::vector<CharPr*>    getCharProperties();
        CharPr*                    getCharProperty(UINT charPrIDRef);
        std::vector<FontFace*>    getFontFaces();
        
    private:

        bool readCharProperties(OWPML::COwpmlDocumnet* document);
        bool readFontFaces(OWPML::COwpmlDocumnet* document);        

        std::vector<CharPr*>    m_charProperties;
        std::vector<FontFace*>    m_fontFaces;
    };

}

