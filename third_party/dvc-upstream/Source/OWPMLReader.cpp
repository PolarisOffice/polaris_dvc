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
#include "OWPMLReader.h"

#include "RCharShape.h"
#include "RParaShape.h"
#include "RTable.h"

#include "ReaderUtil.h"

namespace OWPMLReaderModule {

    OWPMLReader::OWPMLReader(LPCWSTR filePath)
    {
        m_document = OWPML::COwpmlDocumnet::OpenDocument(filePath);

        m_paraShape = NULL;
        m_borderFills = NULL;
        m_charShape = NULL;        
        m_tables = NULL;        
        m_outlineShape = NULL;
        m_bullets = NULL;
    }

    OWPMLReader::~OWPMLReader()
    {
        if (m_charShape)
            delete m_charShape;
        m_charShape = NULL;

        if (m_paraShape)
            delete m_paraShape;
        m_paraShape = NULL;

        if (m_tables)
            delete m_tables;
        m_tables = NULL;

        if (m_borderFills)
            delete m_borderFills;
        m_borderFills = NULL;

        if (m_outlineShape)
            delete m_outlineShape;
        m_outlineShape = NULL;

        if (m_bullets)
            delete m_bullets;
        m_bullets = NULL;

        if (m_runTypeInfos.size() > 0)
        {
            std::vector<RunTypeInfo*>::iterator iter = m_runTypeInfos.begin();

            for (iter; iter != m_runTypeInfos.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;
            }
            m_runTypeInfos.clear();
        }
    }

    RCharShape* OWPMLReader::getCharShape()
    {
        if (!m_document)
            return NULL;

        if (!m_charShape)
            m_charShape = new RCharShape(m_document);
        
        return m_charShape;
    }

    RParaShape* OWPMLReader::getParaShape()
    {
        if (!m_document)
            return NULL;

        if (!m_paraShape)
            m_paraShape = new RParaShape(m_document);

        return m_paraShape;
    }

    RTables* OWPMLReader::getTables()
    {
        if (!m_document)
            return NULL;

        if (!m_tables)
            m_tables = new RTables(m_document);

        return m_tables;
    }

    RBorderFills* OWPMLReader::getBorderFills()
    {
        if (!m_document)
            return NULL;

        if (!m_borderFills)
            m_borderFills = new RBorderFills(m_document);

        return m_borderFills;
    }

    ROutlineShape*    OWPMLReader::getOutlineShape()
    {
        if (!m_document)
            return NULL;

        if (!m_outlineShape)
            m_outlineShape = new ROutlineShape(m_document);

        return m_outlineShape;
    }

    RBullets*    OWPMLReader::getBullets()
    {
        if (!m_document)
            return NULL;

        if (!m_bullets)
            m_bullets = new RBullets(m_document);

        return m_bullets;
    }

    bool OWPMLReader::haveMacroInDocument()
    {
        if (!m_document)
            return false;

        OWPML::CManifestType* manifest = m_document->GetPackage()->Getmanifest();

        if (!manifest)
            return false;

        for (int i = 0; i < 100; i++)
        {
            OWPML::CItem* item = manifest->Getitem(i);

            if (!item)
                return false;

            std::wstring str = item->GetHref();

            if (str.find(L".js", 3) != std::wstring::npos)
                return true;
        }

        return false;        
    }


    // CharShape -------------------------
    LPCWSTR OWPMLReader::getFontFace(UINT prIDRef, bool isCharPrIDRef)
    {
        if (!m_charShape)
            return NULL;
        
        std::vector<CharPr*> charProperties = m_charShape->getCharProperties();
        std::vector<FontFace*> fontFaces = m_charShape->getFontFaces();

        if (charProperties.size() <= 0 || fontFaces.size() <= 0)
            return NULL;

        if (isCharPrIDRef)
        {
            std::vector<CharPr*>::iterator iter = charProperties.begin();

            for (iter; iter != charProperties.end(); iter++)
            {
                if ((*iter)->charPrID == prIDRef)
                {
                    // Hangul만 하기 때문에
                    FontFace* face = fontFaces.at(0);

                    std::map<UINT, LPCWSTR>::iterator fontIter = face->fonts.begin();
                    for (fontIter; fontIter != face->fonts.end(); fontIter++)
                    {
                        if ((*fontIter).first == (*iter)->charPrInfo.fontRef)
                            return (*fontIter).second;
                    }
                }
            }
        }
        else
        {
            std::vector<FontFace*>::iterator iter = fontFaces.begin();

            for (iter; iter != fontFaces.end(); iter++)
            {
                std::map<UINT, LPCWSTR>::iterator fontIter = (*iter)->fonts.begin();

                for (fontIter; fontIter != (*iter)->fonts.end(); fontIter++)
                {
                    if ((*fontIter).first == prIDRef)
                        return (*fontIter).second;
                }
            }
        }

        return NULL;
    }




    std::vector<RunTypeInfo*> OWPMLReader::GetRunTypeInfos()
    {
        if (m_runTypeInfos.size() <= 0)
        {
            std::vector<OWPML::CSectionType*>* sectionTypes = m_document->GetSections();
            UINT pageNo = 0;
            UINT lineNo = 1;

            for (size_t index = 0; index < sectionTypes->size(); index++)
            {
                if (!GetPageInfo(index))    // 섹션마다 page의 정보가 다를수가 있기 때문에 다시 읽어들인다.
                    return m_runTypeInfos;
                    
                OWPML::CSectionType* sectionType = sectionTypes->at(index);
                (OWPML::CPType*)FindPageInfo(sectionType, ID_PARA_PType, pageNo, lineNo);
            }            
        }

        return m_runTypeInfos;
    }    

    OWPML::CObject* OWPMLReader::FindPageInfo(OWPML::CObject* object, UINT refID, UINT& pageNo, UINT& lineNo)
    {
        OWPML::Objectlist* list = object->GetObjectList();

        if (!list)
            return NULL;

        size_t nCnt = list->size();
        nCnt;

        OWPML::CObject* obj = NULL;
        OWPML::CObject* resultObj = NULL;

        UINT beforeVertPos = 0;
        UINT beforeVertSize = 0;

        size_t i = 0;
        for (i = 0; i < nCnt; i++)
        {
            obj = list->at(i);

            if (obj) {
                LPCWSTR eleName = obj->GetElemName();
                eleName;
                if (refID != obj->GetID()) {
                    resultObj = FindPageInfo(obj, refID, pageNo, lineNo);
                    if (resultObj)
                        return resultObj;
                }
                else
                {
                    if (obj->GetID() != ID_PARA_PType)
                        return NULL;

                    OWPML::CPType* pPType = (OWPML::CPType*)obj;

                    OWPML::CLineSegArray* pSegArray = pPType->Getlinesegarray();
                    int nSegCnt = pSegArray->GetChildCount();

                    OWPML::CLineSeg* pLineSeg = NULL;
                                                
                    pLineSeg = pSegArray->GetLineSeg(0);
                    if ((pLineSeg->GetVertPos() == 0 && pPType->GetParentObj() && pPType->GetParentObj()->GetID() == ID_BODY_SectionType)
                        || (pPType->GetParentObj() && pPType->GetParentObj()->GetID() == ID_BODY_SectionType && pLineSeg->GetVertPos() < beforeVertPos + beforeVertSize))
                    {
                        pageNo++;
                        lineNo = 1;
                    }
                                        
                    UINT            prarPrIDRef = pPType->GetParaPrIDRef();
                    OWPML::CTc*        pTc = (OWPML::CTc*)isInTable(pPType);
                    bool            IsInObj = isInObject(pPType);
                    bool            isStyleType = false;
                    UINT            outlineShapeIDRef = 0;
                    bool            isInTableInTable = false;

                    if (pPType->GetStyleIDRef() != 0)
                        isStyleType = true;

                    OWPML::CTableType* tableType = (OWPML::CTableType*)ReaderUtil::FindObjectFromParents(pPType, ID_PARA_TableType);
                    OWPML::CTableType* tableType2 = NULL;
                    if (tableType)
                        tableType2 = (OWPML::CTableType*)ReaderUtil::FindObjectFromParents(tableType, ID_PARA_TableType);
                    if (tableType2)
                        isInTableInTable = true;

                    OWPML::CSectionDefinitionType* secPr = (OWPML::CSectionDefinitionType*)ReaderUtil::FindObjectFromParents(pPType, ID_PARA_SectionDefinitionType, true);
                    if (secPr)
                        outlineShapeIDRef = secPr->GetOutlineShapeIDRef();


                    int nCnt = 0;
                    for (int i = 0; i < pPType->GetChildCount(); i++)
                    {
                        OWPML::CRunType* pRunType = pPType->Getrun(i);

                        if (pRunType && pRunType->HasChildList())
                        {
                            RunTypeInfo* pInfo = new RunTypeInfo;

                            OWPML::CT* t = pRunType->Gett();

                            if (t)
                            {
                                pInfo->text = ReaderUtil::ProcessingTextElement(t);
                            }
                            pInfo->charPrIDRef = pRunType->GetCharPrIDRef();
                            pInfo->paraPrIDRef = prarPrIDRef;

                            pInfo->lineNo = lineNo;
                            pInfo->pageNo = pageNo;                            

                            if (pTc)
                            {
                                pInfo->isInTable = true;
                                pInfo->cellInfo.col = pTc->GetcellAddr()->GetColAddr();
                                pInfo->cellInfo.row = pTc->GetcellAddr()->GetRowAddr();
                                OWPML::CTableType* pTableType = (OWPML::CTableType*)pTc->GetParentObj()->GetParentObj();
                                pInfo->tableID = pTableType->GetId();            

                                pInfo->isInTableInTable = isInTableInTable;
                            }
                            else if (IsInObj)
                            {
                                pInfo->isInShape = true;
                            }
                            else
                            {
                                OWPML::CFieldBegin* begin = (OWPML::CFieldBegin*)ReaderUtil::FindObject(pRunType->GetParentObj(), ID_PARA_FieldBegin);

                                if (begin && begin->GetType() == OWPML::FIELDTYPE::FT_HYPERLINK)
                                    pInfo->isHyperlink = true;                                                    
                            }

                            pInfo->isStyle = isStyleType;

                            OWPML::CSectionDefinitionType* secPr = (OWPML::CSectionDefinitionType*)ReaderUtil::FindObjectFromParents(pPType, ID_PARA_SectionDefinitionType, true);
                            if (secPr)
                                pInfo->outlineShapeIDRef = secPr->GetOutlineShapeIDRef();

                            m_runTypeInfos.push_back(pInfo);
                        }
                    }        
                    
                    resultObj = FindPageInfo(obj, refID, pageNo, lineNo);

                    if (!pTc && !IsInObj)
                    {
                        lineNo += nSegCnt;    // 다음 라인수                            
                        beforeVertPos = pSegArray->GetLineSeg(nSegCnt - 1)->GetVertPos();
                        beforeVertSize = pSegArray->GetLineSeg(nSegCnt - 1)->GetVertSize();
                    }

                    if (nSegCnt > 1)
                    {
                        for (int index = nSegCnt - 1; index > 0; index--)
                        {
                            if (pSegArray->GetLineSeg(index)->GetVertPos() == 0)
                            {
                                pageNo++;
                                lineNo = nSegCnt - index + 1;                                
                                break;
                            }
                        }                        
                    }                    

                    if (resultObj)
                        return resultObj;
                }
            }
        }

        return NULL;
    }

    bool OWPMLReader::GetPageInfo(UINT index)
    {
        std::vector<OWPML::CSectionType*>* sectionTypes = m_document->GetSections();

        OWPML::CSectionType* sectionType = sectionTypes->at(index);;

        if (!sectionType)
            return false;

        OWPML::CSectionDefinitionType* secPr = sectionType->Getp()->Getrun()->GetsecPr();

        UINT pageWidth = secPr->GetpagePr()->GetWidth();
        UINT pageHeight = secPr->GetpagePr()->GetHeight();
        UINT headerSize = secPr->GetpagePr()->Getmargin()->GetHeader();
        UINT footerSize = secPr->GetpagePr()->Getmargin()->GetFooter();
        UINT leftSize = secPr->GetpagePr()->Getmargin()->GetLeft();
        UINT rightSize = secPr->GetpagePr()->Getmargin()->GetRight();
        UINT topSize = secPr->GetpagePr()->Getmargin()->GetTop();
        UINT bottomSize = secPr->GetpagePr()->Getmargin()->GetBottom();

        UINT usebleWidth = pageWidth - leftSize - rightSize;
        m_usebleHeight = pageHeight - headerSize - footerSize - topSize - bottomSize;

        // lineseg 에서 vertpos의 값이 usebleHeight 를 넘지 않으면 그 값은 해당 페이지 내에 있다는 뜻이다.
        // 즉 lineno를 체크할때 이 값으로 하면 된다.

        return true;
    }

    OWPML::CTc* OWPMLReader::isInTable(OWPML::CPType* pObject)
    {
        bool bResult = false;                

        if (pObject->GetParentObj() && pObject->GetParentObj()->GetParentObj())
        {
            if (pObject->GetParentObj()->GetParentObj()->GetID() == ID_PARA_Tc)
                return (OWPML::CTc*)(pObject->GetParentObj()->GetParentObj());
        }        

        return NULL;
    }

    bool OWPMLReader::isInObject(OWPML::CPType* pObject)
    {
        if (pObject->GetParentObj() && pObject->GetParentObj()->GetParentObj())
        {
            if (pObject->GetParentObj()->GetParentObj()->GetID() == ID_PARA_DrawText)
            {
                return true;
            }
        }

        return false;
    }
}