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
#include "ExportInterface.h"
#include "DVCInterface.h"

class DVCErrorInfo : public DVC::IDVCErrInfo
{
public:
    DVCErrorInfo(UINT errorCode, std::wstring errorString);
    DVCErrorInfo(UINT charPrIDRef, UINT paraPrIDRef, std::wstring text, UINT pageNo, UINT lineNo, UINT errorCode, UINT tableID, bool isInTable, bool isInTableInTable, UINT row, UINT col, bool isInShape, bool useHyperlink, bool useStyle, std::wstring errorString);
    ~DVCErrorInfo() {}

public :
    UINT            getCharPrIDRef() { return m_charPrIDRef; }
    UINT            getParaPrIDRef() { return m_paraPrIDRef; }
    std::wstring    getText() { return m_text; }
    UINT            getPageNo() { return m_pageNo; }
    UINT            getLineNo() { return m_lineNo; }
    UINT            getErrorCode() { return m_errorCode; }

    bool            getIsInTable() { return m_isInTable; }
    bool            getIsInTableInTable() { return m_isInTableInTable; }
    bool            getIsInShape() { return m_isInShape; }
    UINT            getTableID() { return m_tableID; }
                    
    UINT            getTableRow() { return m_tableRow; }
    UINT            getTableCol() { return m_tableCol; }

    bool            getUseHyperlink() { return m_useHyperlink; }
    bool            getUseStyle() { return m_useStyle; }
                    
    std::wstring    getErrorString() { return m_errorString; }    

    void            setErrorCode(UINT error) { m_errorCode = error; }
    void            setErrorString(std::wstring error) { m_errorString = error; }

private:
    UINT                m_charPrIDRef;
    UINT                m_paraPrIDRef;
    std::wstring        m_text;
    UINT                m_pageNo;
    UINT                m_lineNo;
    UINT                m_errorCode;

    bool                m_isInTable = false;
    bool                m_isInTableInTable = false;
    bool                m_isInShape = false;
    UINT                m_tableID = 0;

    UINT                m_tableRow = 0;
    UINT                m_tableCol = 0;

    bool                m_useHyperlink = false;
    bool                m_useStyle = false;

    std::wstring        m_errorString;
};

