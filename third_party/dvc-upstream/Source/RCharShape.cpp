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
#include "RCharShape.h"
#include "ReaderUtil.h"

namespace OWPMLReaderModule {

    RCharShape::RCharShape(OWPML::COwpmlDocumnet* document)
    {
        if (document)
        {
            readCharProperties(document);
            readFontFaces(document);
        }
    }

    RCharShape::~RCharShape()
    {
        if (m_charProperties.size() > 0)
        {
            std::vector<CharPr*>::iterator iter = m_charProperties.begin();

            for (iter; iter != m_charProperties.end(); iter++)
            {
                if ((*iter))
                    delete *iter;
                *iter = NULL;                
            }
            m_charProperties.clear();
        }

        if (m_fontFaces.size() > 0)
        {
            std::vector<FontFace*>::iterator fontiter = m_fontFaces.begin();

            for (fontiter; fontiter != m_fontFaces.end(); fontiter++)
            {
                if ((*fontiter))
                    delete *fontiter;
                *fontiter = NULL;
            }
            m_fontFaces.clear();
        }
    }

    std::vector<CharPr*> RCharShape::getCharProperties()
    {
        return m_charProperties;
    }

    CharPr* RCharShape::getCharProperty(UINT charPrIDRef)
    {
        if (m_charProperties.size() <= 0)
            return NULL;

        std::vector<CharPr*>::iterator iter = m_charProperties.begin();

        for (iter; iter != m_charProperties.end(); iter++)
        {
            if ((*iter)->charPrID == charPrIDRef)
                return (*iter);            
        }

    }

    std::vector<FontFace*> RCharShape::getFontFaces()
    {
        return m_fontFaces;
    }
    
    bool RCharShape::readCharProperties(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (document == NULL)
            return false;

        OWPML::CHWPMLHeadType* headType = document->GetHead();

        OWPML::CCharProperties* charProperties = (OWPML::CCharProperties*)ReaderUtil::FindObject(headType, ID_HEAD_CharProperties);

        if (charProperties)
        {
            UINT nIndex = 0;
            for (nIndex = 0; nIndex < charProperties->GetChildCount(); nIndex++)
            {
                OWPML::CCharShapeType* charShapeType = charProperties->GetcharPr(nIndex);

                if (charShapeType)
                {
                    CharPr* charPr = new CharPr;

                    charPr->charPrID = charShapeType->GetId();

                    charPr->height = charShapeType->GetHeight();
                    charPr->textColor = charShapeType->GetTextColor();
                    charPr->shadeColor = charShapeType->GetShadeColor();    // enum
                    charPr->useFontSpace = charShapeType->GetUseFontSpace();
                    charPr->useKerning = charShapeType->GetUseKerning();
                    charPr->symMark = charShapeType->GetSymMark();    // enum
                    charPr->borderFillIDRef = charShapeType->GetBorderFillIDRef();

                    charPr->charPrInfo.fontRef = charShapeType->GetfontRef(0)->GetHangul();
                    charPr->charPrInfo.ratio = charShapeType->Getratio()->GetHangul();
                    charPr->charPrInfo.spacing = charShapeType->Getspacing()->GetHangul();
                    charPr->charPrInfo.relSz = charShapeType->GetrelSz()->GetHangul();
                    charPr->charPrInfo.offset = charShapeType->Getoffset()->GetHangul();

                    charPr->charPrInfo.bold = charShapeType->Getbold() ? true : false;
                    charPr->charPrInfo.italic = charShapeType->Getitalic() ? true : false;
                    charPr->charPrInfo.underline = charShapeType->Getunderline() ? true : false;
                    charPr->charPrInfo.strikeout = charShapeType->Getstrikeout() ? true : false;
                    charPr->charPrInfo.outline = charShapeType->Getoutline() ? true : false;
                    charPr->charPrInfo.emboss = charShapeType->Getemboss() ? true : false;
                    charPr->charPrInfo.engrave = charShapeType->Getengrave() ? true : false;
                    charPr->charPrInfo.shadow = false;
                    if (charShapeType->Getshadow())
                    {
                        OWPML::CCharShadow* shadow = charShapeType->Getshadow();
                        charPr->charPrInfo.shadow = true;
                        charPr->charPrInfo.shadowType = shadow->GetType();
                        charPr->charPrInfo.shadowX = shadow->GetOffsetX();
                        charPr->charPrInfo.shadowY = shadow->GetOffsetY();
                        charPr->charPrInfo.shadowColor = shadow->GetColor();
                    }
                    charPr->charPrInfo.supscript = charShapeType->Getsupscript() ? true : false;
                    charPr->charPrInfo.subscript = charShapeType->Getsubscript() ? true : false;

                    m_charProperties.push_back(charPr);
                }
            }
            bResult = true;
        }

        return bResult;
    }

    bool RCharShape::readFontFaces(OWPML::COwpmlDocumnet* document)
    {
        bool bResult = false;

        if (document == NULL)
            return false;

        OWPML::CHWPMLHeadType* headType = document->GetHead();

        OWPML::CFontfaces* fontfaces = (OWPML::CFontfaces*)ReaderUtil::FindObject(headType, ID_HEAD_Fontfaces);

        if (fontfaces)    // Header에 선언되어 있는 fontface를 모두 가지고 와서 위 id와 일치하는 정보를 찾고 fontface 이름을 찾자.
        {
            UINT i = 0;
            for (i = 0; i < fontfaces->GetChildCount(); i++)
            {
                OWPML::CFontfaceType* fontfaceType = fontfaces->Getfontface(i);

                if (fontfaceType)
                {
                    FontFace* fontFace = new FontFace;

                    fontFace->fontfaceType = fontfaceType->GetLang();

                    for (int j = 0; j < fontfaceType->GetFontCnt(); j++)
                    {
                        OWPML::CFont* font = fontfaceType->Getfont(j);
                        if (font)
                        {
                            UINT id = font->GetId();
                            fontFace->fonts.insert(std::map<UINT, LPCWSTR>::value_type((UINT)font->GetId(), font->GetFace()));
                        }
                    }

                    m_fontFaces.push_back(fontFace);
                }
            }
        }

        return bResult;
    }
}