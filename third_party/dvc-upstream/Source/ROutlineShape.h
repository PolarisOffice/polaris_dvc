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

    typedef struct SecPr {
        LPCWSTR        id;
        UINT        outlineShapeIDRef;
    };

    typedef struct ParaHead {
        UINT                    start;
        UINT                    level;
        OWPML::PHALIGNTYPE        align;
        bool                    useInstWidth;
        bool                    autoIndent;
        bool                    widthAdjust;
        UINT                    textOffset;
        OWPML::NUMBERTYPE        numType;
        UINT                    charPrIDRef;
        bool                    checkable;
        std::wstring            numFormat;
    };

    class RNumbering 
    {
    public:
        RNumbering(OWPML::CNumberingType* numbering);
        ~RNumbering();

    public:
        std::vector<ParaHead*>        getParaHeads();
        ParaHead*                    getParaHeadByLevel(UINT level); 
        UINT                        getNumberingID();
        UINT                        getStart();

    private:
        bool    readNumberings(OWPML::CNumberingType* numbering);

        std::vector<ParaHead*>        m_paraHeads;
        UINT                        m_numberingID;
        UINT                        m_start;

    };

    class ROutlineShape
    {
    public:
        ROutlineShape(OWPML::COwpmlDocumnet* document);
        ~ROutlineShape();

    public:

        std::vector<SecPr*>            getSectionProperties();
        std::vector<RNumbering*>    getNumberings();
        RNumbering*                    getNumbering(UINT outlineShapeIDRef);

    private:
        bool    readSectionProperties(OWPML::COwpmlDocumnet* document);
        bool    readNumberings(OWPML::COwpmlDocumnet* document);

        std::vector<SecPr*>            m_secPrs;
        std::vector<RNumbering*>    m_numberings;        
    };

}