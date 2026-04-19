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


#include "RCharShape.h"
#include "RParaShape.h"
#include "RTable.h"
#include "ROutlineShape.h"
#include "RBullets.h"

namespace OWPMLReaderModule {
    typedef struct CellInfo {
        UINT        row = 0;
        UINT        col = 0;
    };

    typedef struct RunTypeInfo {
        UINT            charPrIDRef = 0;
        UINT            paraPrIDRef = 0;
        std::wstring    text;
        UINT            pageNo = 0;
        UINT            lineNo = 0;

        bool            isInTable = false;
        bool            isInTableInTable = false;
        bool            isInShape = false;
        UINT            tableID = 0;
        CellInfo        cellInfo;

        UINT            outlineShapeIDRef = 0;

        bool            isHyperlink = false;
        bool            isStyle = false;
        
        UINT            errorCode = 0;
        std::wstring    errorString;
    };

    class OWPMLReader
    {
    public:
        OWPMLReader(LPCWSTR filePath);
        ~OWPMLReader();

    public:

        RCharShape*        getCharShape();
        RParaShape*        getParaShape();
        RTables*        getTables();
        RBorderFills*    getBorderFills();
        ROutlineShape*    getOutlineShape();
        RBullets*        getBullets();

        bool            haveMacroInDocument();

        // charshape
        LPCWSTR            getFontFace(UINT prIDRef, bool isCharPrIDRef = false);
               
        std::vector<RunTypeInfo*> GetRunTypeInfos();

    private:        
        OWPML::CObject* FindPageInfo(OWPML::CObject* object, UINT refID, UINT& pageNo, UINT& lineNo);

        OWPML::CTc* isInTable(OWPML::CPType* pObject);
        bool isInObject(OWPML::CPType* pObject);

        bool GetPageInfo(UINT index);

        std::vector<RunTypeInfo*> m_runTypeInfos;
        OWPML::COwpmlDocumnet* m_document;

        RCharShape*            m_charShape;
        RParaShape*            m_paraShape;
        RTables*            m_tables;
        RBorderFills*        m_borderFills;
                
        ROutlineShape*        m_outlineShape;
        RBullets*            m_bullets;

        UINT                m_usebleHeight;    // page에서 실제 사용할수 있는 영역

    };

}