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

#include "pch.h"
#include "DVCErrorInfo.h"

DVCErrorInfo::DVCErrorInfo(UINT errorCode, std::wstring errorString)
{
    m_charPrIDRef = 0;
    m_paraPrIDRef = 0;
    m_text = L"";
    m_pageNo = 0;
    m_lineNo = 0;
    m_errorCode = errorCode;

    m_isInTable = false;
    m_isInTableInTable = false;
    m_isInShape = false;
    m_tableID = 0;

    m_tableRow = 0;
    m_tableCol = 0;

    m_useHyperlink = false;
    m_useStyle = false;

    m_errorString = errorString;
}

DVCErrorInfo::DVCErrorInfo(UINT charPrIDRef, UINT paraPrIDRef, std::wstring text, UINT pageNo, UINT lineNo, UINT errorCode,
    UINT tableID, bool isInTable, bool isInTableInTable, UINT row, UINT col, bool isInShape, bool useHyperlink, bool useStyle, std::wstring errorString)
{
    m_charPrIDRef = charPrIDRef;
    m_paraPrIDRef = paraPrIDRef;
    m_text = text;
    m_pageNo = pageNo;
    m_lineNo = lineNo;
    m_errorCode = errorCode;

    m_tableID = tableID;
    m_isInTable = isInTable;
    m_isInTableInTable = isInTableInTable;
    m_tableRow = row;
    m_tableCol = col;
    m_isInShape = isInShape;
    m_useHyperlink = useHyperlink;
    m_useStyle = useStyle;
    m_errorString = errorString;
}
