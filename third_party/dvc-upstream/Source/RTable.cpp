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
#include "RTable.h"

#include "ReaderUtil.h"

namespace OWPMLReaderModule
{
    RBorderFills::RBorderFills(OWPML::COwpmlDocumnet* document)
    {
        readBorderFills(document);
    }

    RBorderFills::~RBorderFills()
    {
        std::vector<BorderFill*> m_borderFills;
    }

    std::vector<BorderFill*> RBorderFills::getBorderFills()
    {
        return m_borderFills;
    }

    BorderFill* RBorderFills::getBorderFill(UINT borderFillIDRef)
    {
        if (m_borderFills.size() <= 0)
            return NULL;

        std::vector<BorderFill*>::iterator iter = m_borderFills.begin();

        for (iter; iter != m_borderFills.end(); iter++)
        {
            if ((*iter)->id == borderFillIDRef)
                return (*iter);
        }

        return NULL;
    }

    void RBorderFills::readBorderFills(OWPML::COwpmlDocumnet* document)
    {
        OWPML::CHWPMLHeadType* headerType = document->GetHead();

        if (headerType)
        {
            OWPML::CBorderFills* borderFills = (OWPML::CBorderFills*)ReaderUtil::FindObject(headerType, ID_HEAD_BorderFills);

            if (borderFills)
            {
                int nCnt = borderFills->GetChildCount();

                for (int i = 0; i < nCnt; i++)
                {
                    OWPML::CBorderFillType* fillType = borderFills->GetborderFill(i);

                    if (fillType)
                    {
                        BorderFill* borderFill = new BorderFill;
                        
                        borderFill->id = fillType->GetId();
                        borderFill->TreeD = fillType->GetThreeD();
                        borderFill->shadow = fillType->GetShadow();
                        borderFill->centerLine = fillType->GetCenterLine();
                        borderFill->breakCellSeperateLine = fillType->GetBreakCellSeparateLine();
                        
                        if (fillType->Getslash())
                        {
                            borderFill->slashType = fillType->Getslash()->GetType();
                            borderFill->slashcrooked = fillType->Getslash()->GetCrooked();
                            borderFill->slashIsCounter = fillType->Getslash()->GetIsCounter();
                        }
                        
                        if (fillType->GetbackSlash())
                        {
                            borderFill->backslashSlashType = fillType->GetbackSlash()->GetType();
                            borderFill->backslashCrooked = fillType->GetbackSlash()->GetCrooked();
                            borderFill->backslashIsCounter = fillType->GetbackSlash()->GetIsCounter();
                        }
                        
                        if (fillType->GetleftBorder())
                        {
                            borderFill->leftType = fillType->GetleftBorder()->GetType();
                            borderFill->leftWidth = convertBorderWidth(fillType->GetleftBorder()->GetWidth());
                            borderFill->leftColor = fillType->GetleftBorder()->GetColor();
                        }
                        
                        if (fillType->GetrightBorder())
                        {
                            borderFill->rightType = fillType->GetrightBorder()->GetType();
                            borderFill->rightWidth = convertBorderWidth(fillType->GetrightBorder()->GetWidth());
                            borderFill->rightColor = fillType->GetrightBorder()->GetColor();
                        }
                        
                        if (fillType->GettopBorder())
                        {
                            borderFill->topType = fillType->GettopBorder()->GetType();
                            borderFill->topWidth = convertBorderWidth(fillType->GettopBorder()->GetWidth());
                            borderFill->topColor = fillType->GettopBorder()->GetColor();
                        }
                        
                        if (fillType->GetbottomBorder())
                        {
                            borderFill->bottomType = fillType->GetbottomBorder()->GetType();
                            borderFill->bottomWidth = convertBorderWidth(fillType->GetbottomBorder()->GetWidth());
                            borderFill->bottomColor = fillType->GetbottomBorder()->GetColor();
                        }
                        
                        if (fillType->Getdiagonal())
                        {
                            borderFill->diagonalType = fillType->Getdiagonal()->GetType();
                            borderFill->diagonalWidth = convertBorderWidth(fillType->Getdiagonal()->GetWidth());
                            borderFill->diagonalColor = fillType->Getdiagonal()->GetColor();
                        }

                        m_borderFills.push_back(borderFill);
                    }
                }
            }
        }
    }

    float RBorderFills::convertBorderWidth(OWPML::LINEWITHTYPE width)
    {
        switch (width)
        {
        case OWPML::LWT_0_1:
            return 0.1f;
        case OWPML::LWT_0_12:
            return 0.12f;
        case OWPML::LWT_0_15:
            return 0.15f;
        case OWPML::LWT_0_2:
            return 0.2f;
        case OWPML::LWT_0_25:
            return 0.25f;
        case OWPML::LWT_0_3:
            return 0.3f;
        case OWPML::LWT_0_4:
            return 0.4f;
        case OWPML::LWT_0_5:
            return 0.5f;
        case OWPML::LWT_0_6:
            return 0.6f;
        case OWPML::LWT_0_7:
            return 0.7f;
        case OWPML::LWT_1_0:
            return 1.0f;
        case OWPML::LWT_1_5:
            return 1.5f;
        case OWPML::LWT_2_0:
            return 2.0f;
        case OWPML::LWT_3_0:
            return 3.0f;
        case OWPML::LWT_4_0:
            return 4.0f;
        case OWPML::LWT_5_0:
            return 5.0f;        
        }

        return 0.0f;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    RTableColumn::RTableColumn(OWPML::CTc* tc)
    {
        if (tc)
            readTableColumn(tc);
    }

    RTableColumn::~RTableColumn()
    {        
        m_callAddr.clear();
    }

    std::wstring RTableColumn::getStrName()
    {
        return m_strName;
    }

    bool RTableColumn::getHeader()
    {
        return m_header;
    }

    bool RTableColumn::getGasMargin()
    {
        return m_gasMargin;
    }

    bool RTableColumn::getProtect()
    {
        return m_protect;
    }

    bool RTableColumn::getEditable()
    {
        return m_editable;
    }

    bool RTableColumn::getDirty()
    {
        return m_dirty;
    }

    UINT16 RTableColumn::getBorderFillIDRef()
    {
        return m_borderFillIDRef;
    }

    bool RTableColumn::haveTableInTable()
    {
        return m_tableInTable;
    }

    UINT RTableColumn::getCol()
    {
        return m_col;
    }

    UINT RTableColumn::getRow()
    {
        return m_row;
    }


    bool RTableColumn::readTableColumn(OWPML::CTc* tc)
    {
        bool bResult = false;

        if (!tc)
            return bResult;

        m_strName = tc->GetName();
        m_header = tc->GetHeader();
        m_gasMargin = tc->GetHasMargin();
        m_protect = tc->GetProtect();
        m_editable = tc->GetEditable();
        m_dirty = tc->GetDirty();
        m_borderFillIDRef = tc->GetBorderFillIDRef();        
        
        return getObject(tc);
    }

    bool RTableColumn::getObject(OWPML::CObject* object)
    {
        if (!object)
            return false;

        OWPML::Objectlist* pList = object->GetObjectList();
        OWPML::CObject* pObject = NULL;

        UINT count = 0;

        if (!pList)
            return false;

        count = (UINT)pList->size();

        for (size_t i = 0; i < count; i++)
        {
            pObject = pList->at(i);
            if (pObject)
            {
                UINT uID = pObject->GetID();
                switch (uID)
                {
                case ID_PARA_CellAddr:
                {
                    OWPML::CCellAddr* addr = (OWPML::CCellAddr*)pObject;
                    m_callAddr.insert(std::map<INT, INT>::value_type(addr->GetColAddr(), addr->GetRowAddr()));
                    m_col = addr->GetColAddr();
                    m_row = addr->GetRowAddr();
                    break;
                }
                case ID_PARA_CellSpan:
                    // 추후 작업
                    break;
                case ID_PARA_CellSz:
                    // 추후 작업
                    break;
                case ID_PARA_ParaListType:
                    //RSubList;

                    break;
                default:
                    break;
                }
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    RTableRow::RTableRow(OWPML::CTr* tr)
    {
        if (tr)
            readTableRow(tr);
    }

    RTableRow::~RTableRow()
    {
        if (m_columns.size() > 0)
        {
            std::vector<RTableColumn*>::iterator iter = m_columns.begin();

            for (iter; iter != m_columns.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_columns.clear();
        }
    }

    std::vector<RTableColumn*>    RTableRow::getColumns()
    {
        return m_columns;
    }

    bool RTableRow::readTableRow(OWPML::CTr* tr)
    {
        bool bResult = false;

        if (!tr)
            return bResult;

        return getObject(tr);
    }

    bool RTableRow::getObject(OWPML::CObject* object)
    {
        if (!object)
            return false;

        OWPML::Objectlist* pList = object->GetObjectList();
        OWPML::CObject* pObject = NULL;

        UINT count = 0;

        if (!pList)
            return false;
                
        count = (UINT)pList->size();

        for (size_t i = 0; i < count; i++)
        {
            pObject = pList->at(i);
            if (pObject)
            {
                UINT uID = pObject->GetID();
                switch (uID)
                {
                case ID_PARA_Tc:
                {
                    OWPML::CTc* pTr = (OWPML::CTc*)pObject;
                    RTableColumn* tableColumn = new RTableColumn(pTr);
                    m_columns.push_back(tableColumn);
                    break;
                }
                default:
                    break;
                }
            }
        }        

        return true;
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    RTable::RTable(OWPML::CTableType* tableType)
    {
        if (tableType)
            readTable(tableType);
    }

    RTable::~RTable()
    {        
        if (m_tableRows.size() > 0)
        {
            std::vector<RTableRow*>::iterator iter = m_tableRows.begin();

            for (iter; iter != m_tableRows.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_tableRows.clear();
        }
    }
    
    std::vector<RTableRow*>    RTable::getTableRows()
    {
        return m_tableRows;
    }
    
    bool RTable::readTable(OWPML::CTableType* tableType)
    {
        bool bResult = false;

        if (!tableType)
            return bResult;

        m_id = tableType->GetId();
        m_zOrder = tableType->GetZOrder();
        m_numberingType = tableType->GetNumberingType();
        m_textWrap = tableType->GetTextWrap();
        m_textFlow = tableType->GetTextFlow();
        m_lock = tableType->GetLock();
        m_dropcapstyle = tableType->GetDropCapStyle();
        m_pageBreak = tableType->GetPageBreak();
        m_repeatHeader = tableType->GetRepeatHeader();
        m_rowCnt = tableType->GetRowCnt();
        m_colCnt = tableType->GetColCnt();
        m_cellSpacing = tableType->GetCellSpacing();
        m_borderFillIDRef = tableType->GetBorderFillIDRef();
        m_noAdjust = tableType->GetNoAdjust();        

        setTableInTable(tableType);

        return getObject(tableType);
    }

    bool RTable::getObject(OWPML::CObject* object)
    {
        if (!object)
            return false;

        OWPML::Objectlist* pList = object->GetObjectList();
        OWPML::CObject* pObject = NULL;

        UINT count = 0;

        if (pList)
        {
            count = (UINT)pList->size();

            for (size_t i = 0; i < count; i++)
            {
                pObject = pList->at(i);
                if (pObject)
                {
                    UINT uID = pObject->GetID();
                    switch (uID)
                    {
                    case ID_PARA_InsideMarginType:
                    {
                        OWPML::CInsideMarginType* pInType = (OWPML::CInsideMarginType*)pObject;
                        m_inLeft = pInType->GetLeft();
                        m_inRight = pInType->GetRight();
                        m_inTop = pInType->GetTop();
                        m_inBottom = pInType->GetBottom();
                        break;
                    }
                    case ID_PARA_OutMargin:
                    {
                        OWPML::COutMargin* pOutType = (OWPML::COutMargin*)pObject;
                        m_outLeft = pOutType->GetLeft();
                        m_outRight = pOutType->GetRight();
                        m_outTop = pOutType->GetTop();
                        m_outBottom = pOutType->GetBottom();
                        break;
                    }
                    case ID_PARA_ASOPos:
                    {
                        OWPML::CASOPos* pPos = (OWPML::CASOPos*)pObject;
                        m_treatAsChar = pPos->GetTreatAsChar();
                        m_affectLSpacing = pPos->GetAffectLSpacing();
                        m_flowWithText = pPos->GetFlowWithText();
                        m_allowOverlap = pPos->GetAllowOverlap();
                        m_holdAnchorAndSO = pPos->GetHoldAnchorAndSO();
                        m_vertRelTo = pPos->GetVertRelTo();
                        m_horzRelTo = pPos->GetHorzRelTo();
                        m_vertAlign = pPos->GetVertAlign();
                        m_horzAlign = pPos->GetHorzAlign();
                        m_vertOffset = pPos->GetVertOffset();
                        m_horzOffset = pPos->GetHorzOffset();
                        break;
                    }
                    case ID_PARA_Sz:
                    {
                        OWPML::CSz* pSz = (OWPML::CSz*)pObject;
                        m_width = pSz->GetWidth();
                        m_widthRelTo = pSz->GetWidthRelTo();
                        m_height = pSz->GetHeight();
                        m_heightRelTo = pSz->GetHeightRelTo();
                        m_protect = pSz->GetProtect();
                        break;
                    }
                    case ID_PARA_Tr:
                    {
                        OWPML::CTr* pTr = (OWPML::CTr*)pObject;
                        RTableRow* tableRow = new RTableRow(pTr);
                        m_tableRows.push_back(tableRow);
                        break;
                    }
                    default:
                        break;
                    }
                }
            }
        }
        else
            return false;

        return true;
    }

    bool RTable::setTableInTable(OWPML::CTableType* tableType)
    {
        if (ReaderUtil::FindObjectFromParents(tableType, ID_PARA_TableType))
            m_tableInTable = true;
        else
            m_tableInTable = false;

        return true;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    RTables::RTables(OWPML::COwpmlDocumnet* document)
    {
        if (document)
            readTables(document);
    }

    std::vector<RTable*> RTables::getTables()
    {
        return m_tables;
    }

    bool RTables::checkBorders(BorderInfo* info)
    {
        std::vector<RTable*>::iterator iter = m_tables.begin();
        for (iter; iter != m_tables.end(); iter++)
        {
            std::vector<RTableRow*> rows = (*iter)->getTableRows();

            std::vector<RTableRow*>::iterator rowIter = rows.begin();
            for (rows; rowIter != rows.end(); rowIter++)
            {
                std::vector<RTableColumn*> cols = (*rowIter)->getColumns();

                std::vector<RTableColumn*>::iterator colIter = cols.begin();
                for (cols; colIter != cols.end(); colIter++)
                {
                    (*colIter)->getBorderFillIDRef();
                    (*colIter)->haveTableInTable();
                }
            }
        }

        return true;

    }


    bool RTables::readTables(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (!document)
            return bResult;

        std::vector<OWPML::CSectionType*>* sectionsTypes = document->GetSections();

        std::vector<OWPML::CObject*> obj;

        if (sectionsTypes && sectionsTypes->size() > 0)
        {
            for (size_t index = 0; index < sectionsTypes->size(); index++)
            {
                OWPML::CSectionType* sectionType = sectionsTypes->at(index);
                if (sectionType && sectionType->GetObjectList())
                {
                    for (size_t i = 0; i <= sectionType->GetObjectList()->size(); i++)
                    {                            
                        OWPML::CPType* pType = sectionType->Getp(i);

                        if (pType && pType->GetObjectList())
                        {
                            ReaderUtil::FindObjectAndSaveObj(pType, ID_PARA_TableType, obj);                            
                        }
                    }
                }
            }
        }    

        if (obj.size() > 0)
        {
            for (int i = 0; i < obj.size(); i++)
            {
                OWPML::CTableType* pTableType = (OWPML::CTableType*)obj[i];
                RTable* pTable = new RTable(pTableType);
                m_tables.push_back(pTable);
            }
        }
    }

    bool RTables::getObject(OWPML::CObject* object)
    {
        if (!object)
            return false;

        OWPML::Objectlist* pList = object->GetObjectList();
        OWPML::CObject* pObject = NULL;

        UINT count = 0;
        
        if (pList)
        {
            count = (UINT)pList->size();

            for (size_t i = 0; i < count; i++)
            {
                pObject = pList->at(i);
                if (pObject)
                {
                    UINT uID = pObject->GetID();
                    switch (uID)
                    {
                    case ID_PARA_TableType:
                    {
                        OWPML::CTableType* pTableType = (OWPML::CTableType*)pObject;
                        RTable* pTable = new RTable(pTableType);
                        m_tables.push_back(pTable);
                        break;
                    }
                    default:
                        break;
                    }
                }
            }
        }
        else
            return false;

        return true;
    }
};