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
#include "ROutlineShape.h"

#include "ReaderUtil.h"

namespace OWPMLReaderModule {

    ////////////////////////////////////////////////////////////////////////////////////////////
    RNumbering::RNumbering(OWPML::CNumberingType* numbering)
    {
        readNumberings(numbering);
    }

    RNumbering::~RNumbering()
    {
        if (m_paraHeads.size() > 0)
        {
            std::vector<ParaHead*>::iterator iter = m_paraHeads.begin();

            for (iter; iter != m_paraHeads.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_paraHeads.clear();
        }
    }

    std::vector<ParaHead*> RNumbering::getParaHeads()
    {
        return m_paraHeads;
    }
    
    ParaHead* RNumbering::getParaHeadByLevel(UINT level)
    {
        std::vector<ParaHead*>::iterator iter = m_paraHeads.begin();

        for (iter; iter != m_paraHeads.end(); iter++)
        {
            if ((*iter)->level == level)
                return (*iter);
        }

        return NULL;
    }

    UINT RNumbering::getNumberingID()
    {
        return m_numberingID;
    }

    UINT RNumbering::getStart()
    {
        return m_start;
    }


    bool RNumbering::readNumberings(OWPML::CNumberingType* numbering)
    {
        if (!numbering)
            return false;

        int nCnt = numbering->GetChildCount();

        for (int i = 0; i < nCnt; i++)
        {
            m_numberingID = numbering->GetId();
            m_start = numbering->GetStart();

            OWPML::CParaHeadType2* headType = numbering->GetparaHead(i);

            if (headType)
            {
                ParaHead* paraHead = new ParaHead;                

                paraHead->start = headType->GetStartNumber();
                paraHead->level = headType->GetLevel();
                paraHead->align = headType->GetAlign();
                paraHead->useInstWidth = headType->GetUseInstWidth();
                paraHead->autoIndent = headType->GetAutoIndent();
                paraHead->widthAdjust = headType->GetWidthAdjust();
                paraHead->textOffset = headType->GetTextOffset();
                paraHead->numType = headType->GetNumFormat();
                paraHead->charPrIDRef = headType->GetCharPrIDRef();
                paraHead->checkable = headType->GetCheckable();
                WCHAR value[_MAX_DIR];
                headType->ReadNodeValue(value, _MAX_DIR);
                paraHead->numFormat.assign(value);

                m_paraHeads.push_back(paraHead);
            }            
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////
    ROutlineShape::ROutlineShape(OWPML::COwpmlDocumnet* document)
    {
        readSectionProperties(document);
        readNumberings(document);
    }

    ROutlineShape::~ROutlineShape()
    {
        if (m_secPrs.size() > 0)
        {
            std::vector<SecPr*>::iterator iter = m_secPrs.begin();

            for (iter; iter != m_secPrs.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_secPrs.clear();
        }

        if (m_numberings.size() > 0)
        {
            std::vector<RNumbering*>::iterator numIter = m_numberings.begin();

            for (numIter; numIter != m_numberings.end(); numIter++)
            {
                if ((*numIter))
                    delete *numIter;
                *numIter = NULL;
            }
            m_numberings.clear();
        }
    }

    std::vector<SecPr*>    ROutlineShape::getSectionProperties()
    {
        return m_secPrs;
    }

    std::vector<RNumbering*> ROutlineShape::getNumberings()
    {
        return m_numberings;
    }

    RNumbering* ROutlineShape::getNumbering(UINT outlineShapeIDRef)
    {
        std::vector<RNumbering*>::iterator iter = m_numberings.begin();

        for (iter; iter != m_numberings.end(); iter++)
        {
            if ((*iter)->getNumberingID() == outlineShapeIDRef)
                return (*iter);
        }

        return NULL;
    }


    bool ROutlineShape::readSectionProperties(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (!document)
            return bResult;

        std::vector<OWPML::CSectionType*>* sectionsTypes = document->GetSections();

        if (sectionsTypes && sectionsTypes->size() > 0)
        {
            for (size_t index = 0; index < sectionsTypes->size(); index++)
            {
                OWPML::CSectionType* sectionType = sectionsTypes->at(index);

                SecPr* secPr = new SecPr;

                OWPML::CSectionDefinitionType* type = sectionType->Getp()->Getrun()->GetsecPr();

                secPr->id = type->GetId();
                secPr->outlineShapeIDRef = type->GetOutlineShapeIDRef();

                m_secPrs.push_back(secPr);
            }

            bResult = true;
        }

        return bResult;
    }
    
    bool ROutlineShape::readNumberings(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (!document)
            return bResult;

        OWPML::CHWPMLHeadType* headType = document->GetHead();

        OWPML::CNumberings* numberings = (OWPML::CNumberings*)ReaderUtil::FindObject(headType, ID_HEAD_Numberings);

        if (numberings)
        {
            int nCnt = numberings->GetItemCnt();

            for (int i = 0; i < nCnt; i++)
            {                
                OWPML::CNumberingType* numbering = numberings->Getnumbering(i);

                RNumbering* rNumbering = new RNumbering(numbering);

                m_numberings.push_back(rNumbering);                
            }
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////    
}