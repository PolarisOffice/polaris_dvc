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
#include "RParaShape.h"
#include "ReaderUtil.h"

namespace OWPMLReaderModule {

    RParaShape::RParaShape(OWPML::COwpmlDocumnet* document)
    {
        if (document)
        {
            readParaProperties(document);
            readTabProperties(document);
        }
    }

    RParaShape::~RParaShape()
    {        
        if (m_paraProperties.size() > 0)
        {
            std::vector<ParaPr*>::iterator iter = m_paraProperties.begin();

            for (iter; iter != m_paraProperties.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_paraProperties.clear();
        }

        if (m_tabProperties.size() > 0)
        {
            std::vector<TabPr*>::iterator tabIter = m_tabProperties.begin();

            for (tabIter; tabIter != m_tabProperties.end(); tabIter++)
            {
                if ((*tabIter))
                    delete *tabIter;
                *tabIter = NULL;
            }
            m_tabProperties.clear();
        }
    }

    std::vector<ParaPr*> RParaShape::getParaProperties()
    {
        return m_paraProperties;
    }

    std::vector<TabPr*> RParaShape::getTabProperties()
    {
        return m_tabProperties;
    }

    ParaPr*    RParaShape::getParaPropertyForID(UINT paraID)
    {
        std::vector<ParaPr*>::iterator iter = m_paraProperties.begin();

        for (iter; iter != m_paraProperties.end(); iter++)
        {
            if ((*iter)->paraPrID == paraID)
                return (*iter);
        }

        return NULL;
    }


    bool RParaShape::readParaProperties(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (document == NULL)
            return false;

        OWPML::CHWPMLHeadType* headType = document->GetHead();

        OWPML::CParaProperties* paraProperties = (OWPML::CParaProperties*)ReaderUtil::FindObject(headType, ID_HEAD_ParaProperties);

        if (paraProperties)
        {
            UINT nIndex = 0;
            for (nIndex = 0; nIndex < paraProperties->GetChildCount(); nIndex++)
            {
                OWPML::CParaShapeType* paraShapeType = paraProperties->GetparaPr(nIndex);
                if (paraShapeType)
                {
                    ParaPr* paraPr = new ParaPr;

                    paraPr->paraPrID = paraShapeType->GetId();
                    paraPr->tabPrIDRef = paraShapeType->GetTabPrIDRef();
                    paraPr->condense = paraShapeType->GetCondense();
                    paraPr->fontLineHeight = paraShapeType->GetFontLineHeight();
                    paraPr->snapToGrid = paraShapeType->GetSnapToGrid();
                    paraPr->suppressLineNumbers = paraShapeType->GetSuppressLineNumbers();
                    paraPr->checked = paraShapeType->GetChecked();

                    if (paraShapeType->Getalign())
                    {
                        paraPr->hAlineType = (HAlignType)paraShapeType->Getalign()->GetHorizontal();
                        paraPr->vAlineType = (VAlignType)paraShapeType->Getalign()->GetVertical();
                    }

                    if (paraShapeType->Getheading())
                    {
                        paraPr->headingType = (ParaType)paraShapeType->Getheading()->GetType();
                        paraPr->headingIdRef = paraShapeType->Getheading()->GetIdRef();
                        paraPr->headingLevel = paraShapeType->Getheading()->GetLevel() + 1;    // numbering에서 비교할 때 기준이 1부터 시작하기 때문에
                    }

                    if (paraShapeType->GetbreakSetting())
                    {
                        paraPr->breakLatinWord = (LineBreakLatinWord)paraShapeType->GetbreakSetting()->GetBreakLatinWord();
                        paraPr->breakNonLatinWord = (LineBreakNonLatinWord)paraShapeType->GetbreakSetting()->GetBreakNonLatinWord();
                        paraPr->widowOrphan = paraShapeType->GetbreakSetting()->GetWidowOrphan();
                        paraPr->keepWithNext = paraShapeType->GetbreakSetting()->GetKeepWithNext();
                        paraPr->keepLines = paraShapeType->GetbreakSetting()->GetKeepLines();
                        paraPr->pageBreakBefore = paraShapeType->GetbreakSetting()->GetPageBreakBefore();
                        paraPr->lineWrap = paraShapeType->GetbreakSetting()->GetLineWrap();
                    }

                    if (paraShapeType->GetautoSpacing())
                    {
                        paraPr->autoSpacingEng = paraShapeType->GetautoSpacing()->GetEAsianEng();
                        paraPr->autoSpacingNum = paraShapeType->GetautoSpacing()->GetEAsianNum();
                    }

                    if (paraShapeType->Getmargin())
                    {
                        paraPr->margin.indent = paraShapeType->Getmargin()->Getintent()->GetValue();
                        paraPr->margin.left = paraShapeType->Getmargin()->Getleft()->GetValue();
                        paraPr->margin.right = paraShapeType->Getmargin()->Getright()->GetValue();
                        paraPr->margin.prev = paraShapeType->Getmargin()->Getprev()->GetValue();
                        paraPr->margin.next = paraShapeType->Getmargin()->Getnext()->GetValue();
                    }

                    if (paraShapeType->GetlineSpacing())
                    {
                        paraPr->lineSpacing.type = (LineSpacingType)paraShapeType->GetlineSpacing()->GetType();
                        paraPr->lineSpacing.value = paraShapeType->GetlineSpacing()->GetValue();
                        paraPr->lineSpacing.unit = paraShapeType->GetlineSpacing()->GetUnit();
                    }

                    if (paraShapeType->Getborder())
                    {
                        paraPr->borderFillIDRef = paraShapeType->Getborder()->GetBorderFillIDRef();
                        paraPr->borderOffsetLeft = paraShapeType->Getborder()->GetOffsetLeft();
                        paraPr->borderOffsetRight = paraShapeType->Getborder()->GetOffsetRight();
                        paraPr->borderOffsetTop = paraShapeType->Getborder()->GetOffsetTop();
                        paraPr->borderOffsetBottom = paraShapeType->Getborder()->GetOffsetBottom();
                        paraPr->connect = paraShapeType->Getborder()->GetConnect();
                        paraPr->ignoreMargin = paraShapeType->Getborder()->GetIgnoreMargin();
                    }

                    m_paraProperties.push_back(paraPr);
                }
            }
            bResult = true;
        }

        return bResult;
    }

    bool RParaShape::readTabProperties(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (document == NULL)
            return false;

        OWPML::CHWPMLHeadType* headType = document->GetHead();

        OWPML::CTabProperties* tabProperties = (OWPML::CTabProperties*)ReaderUtil::FindObject(headType, ID_HEAD_TabProperties);

        if (tabProperties)
        {
            UINT nIndex = 0;
            for (nIndex = 0; nIndex < tabProperties->GetChildCount(); nIndex++)
            {
                OWPML::CTabDefType* tabDefType = tabProperties->GettabPr(nIndex);

                if (tabDefType)
                {
                    TabPr* tabPr = new TabPr;

                    tabPr->tabPrID = tabDefType->GetID();
                    tabPr->autoTabLeft = tabDefType->GetAutoTabLeft();
                    tabPr->autoTabRight = tabDefType->GetAutoTabRight();

                    m_tabProperties.push_back(tabPr);
                }
            }
            bResult = true;
        }

        return bResult;
    }
}