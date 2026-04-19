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

    typedef struct BorderFill {
        UINT                id;
        bool                TreeD;
        bool                shadow;
        bool                centerLine;
        bool                breakCellSeperateLine;

        // slash
        OWPML::SLASHTYPE    slashType;
        UINT                slashcrooked;
        bool                slashIsCounter;

        // backslash
        OWPML::SLASHTYPE    backslashSlashType;
        bool                backslashCrooked;
        bool                backslashIsCounter;

        // left
        OWPML::LINETYPE2    leftType;
        float                leftWidth;
        COLORREF            leftColor;

        // right
        OWPML::LINETYPE2    rightType;
        float                rightWidth;
        COLORREF            rightColor;

        // top
        OWPML::LINETYPE2    topType;
        float                topWidth;
        COLORREF            topColor;

        // bottom
        OWPML::LINETYPE2    bottomType;
        float                bottomWidth;
        COLORREF            bottomColor;

        // diagonal
        OWPML::LINETYPE2    diagonalType;
        float                diagonalWidth;
        COLORREF            diagonalColor;

    };

    class RBorderFills {
    public:
        RBorderFills(OWPML::COwpmlDocumnet* document);
        ~RBorderFills();

    public:
        std::vector<BorderFill*>    getBorderFills();
        BorderFill*                    getBorderFill(UINT borderFillIDRef);

    private:
        void    readBorderFills(OWPML::COwpmlDocumnet* document);
        float    convertBorderWidth(OWPML::LINEWITHTYPE width);

        std::vector<BorderFill*> m_borderFills;
    };

//    class RRunType {
//    public:
//        RRunType(OWPML::CRunType* runType) {}
//        ~RRunType();
//
//    public:
//        /*UINT                        getCharPrIDRef();
//        std::vector<std::string>    getTexts();*/
//
//    private:
//        //OWPML::CExtObject*            getObject(UINT uID);
//
//
//        UINT                        charPrIDRef;
//        std::vector<std::string>    strTxt;
//    };
//
//    class RPType {
//        RPType(OWPML::CPType* pType) {}
//        ~RPType() {}
//
//    public:
//        /*UINT                        getParaPrIDRef();
//        UINT                        getStyleIDRef();
//        bool                        getPageBreak();
//        bool                        getColumnBreak();
//        bool                        getMerged();
//                        
//        std::vector<RRunType*>        getRunTables();*/
//
//    private:
//        UINT                        paraPrIDRef;
//        UINT                        styleIDRef;
//        bool                        pageBreak;
//        bool                        columnBreak;
//        bool                        merged;
//
//        std::vector<RRunType*>        runTables;
//    };
//
//
//    class RSubList {
//    public:
//        RSubList(OWPML::CParaListType* paraListType) {}
//        ~RSubList() {}
//
//    public:
//        /*std::string                getSubListID();
//        UINT                    getTextDirection();
//        UINT                    getLineWrap();
//        UINT                    getVertAlign();
//        UINT                    getLinkListIDRef();
//        UINT                    getLinkListNextIDRef();
//        UINT                    getTextWidth();
//        UINT                    getTextHeight();
//        bool                    getHasTextRef();
//        bool                    getHasNumRef();
//
//        std::vector<RPType*>    getPTypes();
//*/
//    private:
//        std::string                strId;
//        UINT                    textDirection;
//        UINT                    lineWrap;
//        UINT                    vertAlign;
//        UINT                    linkListIDRef;
//        UINT                    linkListNextIDRef;
//        UINT                    textWidth;
//        UINT                    textHeight;
//        bool                    hasTextRef;
//        bool                    hasNumRef;
//
//        std::vector<RPType*>    pTypes;
//    };

    class RTableColumn {
    public:
        RTableColumn(OWPML::CTc* tc);
        ~RTableColumn();

    public:
        std::wstring                getStrName();
        bool                        getHeader();
        bool                        getGasMargin();
        bool                        getProtect();
        bool                        getEditable();
        bool                        getDirty();
        UINT16                        getBorderFillIDRef();
        bool                        haveTableInTable();
        UINT                        getCol();
        UINT                        getRow();

    private:
        bool                        readTableColumn(OWPML::CTc* tc);

        bool                        getObject(OWPML::CObject* object);

        std::wstring                m_strName;
        bool                        m_header;
        bool                        m_gasMargin;
        bool                        m_protect;
        bool                        m_editable;
        bool                        m_dirty;
        UINT16                        m_borderFillIDRef;

        std::map<INT, INT>            m_callAddr;
        UINT                        m_col;
        UINT                        m_row;


        bool                        m_tableInTable;
    };

    class RTableRow {
    public:
        RTableRow(OWPML::CTr* tr);
        ~RTableRow();

    public:
        std::vector<RTableColumn*>    getColumns();

    private:
        bool                    readTableRow(OWPML::CTr* tr);

        bool                    getObject(OWPML::CObject* object);

        std::vector<RTableColumn*>    m_columns;

    };

    
    class RTable
    {
    public:
        RTable(OWPML::CTableType* tableType);
        ~RTable();

    public:
        std::vector<RTableRow*>    getTableRows();


        UINT                    getTableID() { return m_id; }
        UINT                    getZOrder() { return m_zOrder; }
        OWPML::ASONUMBERINGTYPE    getNumberingType() { return m_numberingType; }
        OWPML::ASOTEXTWRAPTYPE    getTextWrap() { return m_textWrap; }
        OWPML::ASOTEXTFLOWTYPE    getTextFlow() { return m_textFlow; }
        UINT                    getLock() { return m_lock; }
        UINT                    getDropcapstyle() { return m_dropcapstyle; }
        OWPML::TABLEPAGEBREAKTYPE    getPageBreak() { return m_pageBreak; }
        bool                    getRepeatHeader() { return m_repeatHeader; }
        UINT                    getRowCnt() { return m_rowCnt; }
        UINT                    getColCnt() { return m_colCnt; }
        UINT                    getCellSpacing() { return m_cellSpacing; }
        UINT                    getBorderFillIDRef() { return m_borderFillIDRef; }
        bool                    getNoAdjust() { return m_noAdjust; }

        // sz
        UINT                    getWidth() { return m_width; }
        UINT                    getWidthRelTo() { return m_widthRelTo; }
        UINT                    getHeight() { return m_height; }
        UINT                    getHeightRelTo() { return m_heightRelTo; }
        bool                    getProtect() { return m_protect; }

        // pos
        bool                    getTreatAsChar() { return m_treatAsChar; }
        bool                    getAffectLSpacing() { return m_affectLSpacing; }
        bool                    getFlowWithText() { return m_flowWithText; }
        bool                    getAllowOverlap() { return m_allowOverlap; }
        bool                    getHoldAnchorAndSO() { return m_holdAnchorAndSO; }
        UINT                    getVertRelTo() { return m_vertRelTo; }
        UINT                    getHorzRelTo() { return m_horzRelTo; }
        UINT                    getVertAlign() { return m_vertAlign; }
        UINT                    getHorzAlign() { return m_horzAlign; }
        UINT                    getVertOffset() { return m_vertOffset; }
        UINT                    getHorzOffset() { return m_horzOffset; }

        // outMargin
        UINT                    getOutLeft() { return m_outLeft; }
        UINT                    getOutRight() { return m_outRight; }
        UINT                    getOutTop() { return m_outTop; }
        UINT                    getOutBottom() { return m_outBottom; }

        // inMargin
        UINT                    getInLeft() { return m_inLeft; }
        UINT                    getInRight() { return m_inRight; }
        UINT                    getInTop() { return m_inTop; }
        UINT                    getInBottom() { return m_inBottom; }

        bool                    getTableInTable() { return m_tableInTable; }

    private:
        bool                    readTable(OWPML::CTableType* tableType);

        bool                    getObject(OWPML::CObject* object);

        bool                    setTableInTable(OWPML::CTableType* tableType);

        std::vector<RTableRow*>    m_tableRows;

        UINT                    m_id;
        UINT                    m_zOrder;
        OWPML::ASONUMBERINGTYPE    m_numberingType;
        OWPML::ASOTEXTWRAPTYPE    m_textWrap;
        OWPML::ASOTEXTFLOWTYPE    m_textFlow;
        UINT                    m_lock;
        UINT                    m_dropcapstyle;
        OWPML::TABLEPAGEBREAKTYPE    m_pageBreak;
        bool                    m_repeatHeader;
        UINT                    m_rowCnt;
        UINT                    m_colCnt;
        UINT                    m_cellSpacing;
        UINT                    m_borderFillIDRef;
        bool                    m_noAdjust;
        
        // sz
        UINT                    m_width;
        UINT                    m_widthRelTo;
        UINT                    m_height;
        UINT                    m_heightRelTo;
        bool                    m_protect;

        // pos
        bool                    m_treatAsChar;
        bool                    m_affectLSpacing;
        bool                    m_flowWithText;
        bool                    m_allowOverlap;
        bool                    m_holdAnchorAndSO;
        UINT                    m_vertRelTo;
        UINT                    m_horzRelTo;
        UINT                    m_vertAlign;
        UINT                    m_horzAlign;
        UINT                    m_vertOffset;
        UINT                    m_horzOffset;

        // outMargin
        UINT                    m_outLeft;
        UINT                    m_outRight;
        UINT                    m_outTop;
        UINT                    m_outBottom;

        // inMargin
        UINT                    m_inLeft;
        UINT                    m_inRight;
        UINT                    m_inTop;
        UINT                    m_inBottom;

        bool                    m_tableInTable;
    };
    

    class RTables
    {
    public:
        RTables(OWPML::COwpmlDocumnet* document);
        ~RTables() {};

    public:
        std::vector<RTable*>    getTables();
        RTable*                    getTable(UINT index);        

        bool                    checkBorders(BorderInfo* info);

    private:
        bool                    readTables(OWPML::COwpmlDocumnet* document);

        bool                    getObject(OWPML::CObject* object);

        std::vector<RTable*>    m_tables;
    };





    typedef struct RTable_tmp
    {
        UINT        m_tableID;
        UINT        m_zOrder;
        UINT        m_numberingType;

    };

}

