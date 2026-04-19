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
#include "Checker.h"

#include "Source\JsonModel.h"
#include "Source\OWPMLReader.h"

#include "Util.h"
#include "Source\ReaderUtil.h"
#include "Source\DVCErrorInfo.h"

Checker::Checker(std::wstring checklistFile, std::wstring sourceFile)
{
    m_checkListFile = checklistFile;
    m_sourceFile = sourceFile;    

    m_checkTableDetail = false;    // 기본은 Table 단위로 체크
}

Checker::~Checker()
{
    if (m_owpmlReader)
        delete m_owpmlReader;
    m_owpmlReader = NULL;

    if (m_checkList)
        delete m_checkList;
    m_checkList = NULL;

    ClearErrorInfos();

    if (m_checkedErrorListInfo.size() > 0)
    {
        std::vector<RunTypeInfo*>::iterator iter = m_checkedErrorListInfo.begin();

        for (iter; iter != m_checkedErrorListInfo.end(); iter++)
        {
            if ((*iter))
                delete *iter;
            *iter = NULL;
        }
        m_checkedErrorListInfo.clear();
    }
}

bool Checker::Initialize()
{
    m_checkList = new CheckList(Util::ws2s(m_checkListFile));
    m_owpmlReader = NULL;
    if (m_checkList && m_checkList->parsing())
    {
        m_owpmlReader = new OWPMLReader(m_sourceFile.c_str());

        if (m_owpmlReader)
        {
            m_runTypeInfo = m_owpmlReader->GetRunTypeInfos();

            if (m_runTypeInfo.size() > 0)
                return true;
        }
    }
    
    return false;

}

void Checker::SetOutput(DVC::IDVCOutput* output)
{
    m_dvcOutput = output;
}

bool Checker::CheckCharShape()
{    
    if (!m_owpmlReader)
        return false;

    int nCnt = 0;

    RCharShape* rCharShape = m_owpmlReader->getCharShape();

    std::vector<CharPr*> rCharPr = rCharShape->getCharProperties();

    std::vector<CharPr*>::iterator rCharPrIter = rCharPr.begin();

    bool bResult = false;
    for (rCharPrIter; rCharPrIter != rCharPr.end(); rCharPrIter++)
    {        
        CheckCharShapeToCheckList(*rCharPrIter);        
    }

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();
        
    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_CHAR_SHAPE ||
            (*errorIter)->elementID > JID_CHAR_SHAPE_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->charPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {                    
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;
        
    return true;
}

bool Checker::CheckParaShape()
{
    if (!m_owpmlReader)
        return false;

    int nCnt = 0;

    RParaShape* rParaShape = m_owpmlReader->getParaShape();

    std::vector<ParaPr*> rParaPr = rParaShape->getParaProperties();

    std::vector<ParaPr*>::iterator rParaPrIter = rParaPr.begin();

    bool bResult = false;
    for (rParaPrIter; rParaPrIter != rParaPr.end(); rParaPrIter++)
    {
        CheckParaShapeToCheckList(*rParaPrIter);
    }

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_PARA_SHAPE ||
            (*errorIter)->elementID > JID_PARA_SHAPE_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->paraPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckTable(bool checkDetail)
{
    if (!m_owpmlReader)
        return false;

    m_checkTableDetail = checkDetail;

    int nCnt = 0;

    RTables* rTables = m_owpmlReader->getTables();

    std::vector<RTable*> tables = rTables->getTables();
    
    RBorderFills* borderFills = m_owpmlReader->getBorderFills();

    std::vector<RTable*>::iterator iter = tables.begin();

    bool bResult = false;

    for (iter; iter != tables.end(); iter++)
    {
        CheckTableToCheckList(*iter, borderFills);
    }
                
    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_TABLE ||
            (*errorIter)->elementID > JID_TABLE_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->isInTable && (*runTypeIter)->tableID == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {                
                if (((*runTypeIter)->cellInfo.col == (*errorIter)->col &&
                    (*runTypeIter)->cellInfo.row == (*errorIter)->row) ||
                    (*runTypeIter)->isInTableInTable)
                {
                    m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                    nCnt++;
                }
            }
        }
    }

    ClearErrorInfos();
    
    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckSpacialCharacter()
{
    int nCnt = 0;

    CheckSpacialCharacterToCheckList();

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_SPECIALCHARACTER ||
            (*errorIter)->elementID > JID_SPECIALCHARACTER_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->charPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckOutlineShape()
{
    int nCnt = 0;

    CheckOutlineShapeToCheckList(ParaType::ParaOutline);

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_OUTLINESHAPE ||
            (*errorIter)->elementID > JID_OUTLINESHAPE_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->paraPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckBullet()
{
    int nCnt = 0;

    CheckBulletToCheckList();

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_BULLET ||
            (*errorIter)->elementID > JID_BULLET_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->paraPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckParaNumBullet()
{
    int nCnt = 0;

    // outlineshape 과 거의 동일한 패턴이기 때문에 코드를 같이 사용한다.

    CheckOutlineShapeToCheckList(ParaType::ParaNumber);    // 단 secPr를 사용하지 않기 때문에 이 부분만 제거하고, heading type="NUMBER" 로 바꿔 설정하자.

    if (m_errorInfos.size() == 0)
        return true;

    std::vector<ErrorInfo*>::iterator errorIter = m_errorInfos.begin();

    for (errorIter; errorIter != m_errorInfos.end(); errorIter++)
    {
        if ((*errorIter)->elementID < JID_PARANUMBULLET ||
            (*errorIter)->elementID > JID_PARANUMBULLET_END)
            continue;

        std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
        for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
        {
            if ((*runTypeIter)->paraPrIDRef == (*errorIter)->IDRef)    // 오류가 포함된 charprIDRef를 찾음.
            {
                m_checkedErrorListInfo.push_back(CloneRunInfo(*runTypeIter, (*errorIter)));
                nCnt++;
            }
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckHyperlink()
{
    int nCnt = 0;

    bool useHyperlink = CheckHyperlinkToRunInfo();    // 사용을 하지 말아야 하는 경우에는 false를 리턴한다. 만일 true가 리턴되면 이 체크는 따로 할 필요가 없다.

    if (useHyperlink)
        return true;

    std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
    for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
    {
        if ((*runTypeIter)->isHyperlink)
        {
            RunTypeInfo* info = CloneRunInfo(*runTypeIter);
            info->errorCode = JID_HYPERLINK_PERMISSION;
            info->errorString = Util::s2ws(JIN_HYPERLINK_PERMISSION);
            m_checkedErrorListInfo.push_back(info);
            nCnt++;
        }
    }
    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckStyle()
{
    int nCnt = 0;
    bool useStyle = CheckStyleToRunInfo();    // 사용을 하지 말아야 하는 경우에는 false를 리턴한다. 만일 true가 리턴되면 이 체크는 따로 할 필요가 없다.

    if (useStyle)
        return true;

    std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();
    for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
    {
        if ((*runTypeIter)->isStyle)
        {
            RunTypeInfo* info = CloneRunInfo(*runTypeIter);
            info->errorCode = JID_STYLE_PERMISSION;
            info->errorString = Util::s2ws(JIN_STYLE_PERMISSION);
            m_checkedErrorListInfo.push_back(info);        
            nCnt++;
        }
    }

    ClearErrorInfos();

    if (nCnt)
        return false;

    return true;
}

bool Checker::CheckMacro()
{    
    bool isError = !CheckMacroToCheckLIst();

    if (isError)
    {
        //std::cout << "useMacro : ";
        //std::cout << 0 << std::endl;

        //std::cout << JID_MACRO_PERMISSION << std::endl; // error 항목(ID값)        
        //nCnt++;

        RunTypeInfo* info = new RunTypeInfo;
        info->errorCode = JID_MACRO_PERMISSION;
        info->errorString = Util::s2ws(JIN_MACRO_PERMISSION);
        m_checkedErrorListInfo.push_back(info);
    }
    else
        return true;
    
    return false;
}

bool Checker::PrintCheckedErrorList()
{    
    DVC::IDVCOutput* dvcOutput;

    int nCnt = 0;

    if (m_dvcOutput == NULL)
        return false;

    if (m_checkedErrorListInfo.size() <= 0)
        return false;

    std::vector<RunTypeInfo*>::iterator runTypeIter = m_checkedErrorListInfo.begin();
    for (runTypeIter; runTypeIter != m_checkedErrorListInfo.end(); runTypeIter++)
    {
        DVC::IDVCErrInfo* errorInfo = new DVCErrorInfo((*runTypeIter)->charPrIDRef, (*runTypeIter)->paraPrIDRef, (*runTypeIter)->text, (*runTypeIter)->pageNo, (*runTypeIter)->lineNo, (*runTypeIter)->errorCode,
            (*runTypeIter)->tableID, (*runTypeIter)->isInTable, (*runTypeIter)->isInTableInTable, (*runTypeIter)->cellInfo.row, (*runTypeIter)->cellInfo.col, (*runTypeIter)->isInShape, (*runTypeIter)->isHyperlink, (*runTypeIter)->isStyle, L"");
        
        m_dvcOutput->appendResult(errorInfo);

        //std::wstring text = (*runTypeIter)->text;
        //std::cout << "Text            :    ";

        //HANDLE output = GetStdHandle(STD_OUTPUT_HANDLE);
        //DWORD dwWritten = 0;
        //WriteConsoleW(output, text.c_str(), text.length(), &dwWritten, NULL);
        //
        //std::cout << std::endl;
        //std::cout << "pageNo            :    ";
        //std::cout << (*runTypeIter)->pageNo << std::endl;
        //std::cout << "lineNo            :    ";
        //std::cout << (*runTypeIter)->lineNo << std::endl;

        //std::cout << "isInTable        :    ";
        //std::cout << (*runTypeIter)->isInTable << std::endl;
        //std::cout << "isInTableInTable    :    ";
        //std::cout << (*runTypeIter)->isInTableInTable << std::endl;
        //std::cout << "isInShape        :    ";
        //std::cout << (*runTypeIter)->isInShape << std::endl;
        //std::cout << "tableID            :    ";
        //std::cout << (*runTypeIter)->tableID << std::endl;

        //std::cout << "row            :    ";
        //std::cout << (*runTypeIter)->cellInfo.row << std::endl;
        //std::cout << "col            :    ";
        //std::cout << (*runTypeIter)->cellInfo.col << std::endl;

        //std::cout << "useHyperlink        :    ";
        //std::cout << (*runTypeIter)->isHyperlink << std::endl;
        //std::cout << "useStyle        :    ";
        //std::cout << (*runTypeIter)->isStyle << std::endl;
        //
        //std::cout << "errorCode        :    ";
        //std::cout << (*runTypeIter)->errorCode << std::endl; // error 항목(ID값)        
        //std::cout << std::endl;
    }

    bool isError = !CheckMacroToCheckLIst();
    if (isError)
    {
        /*std::cout << std::endl;
        std::cout << "useMacro            : ";
        std::cout << 0 << std::endl;*/

        DVC::IDVCErrInfo* errorInfo = new DVCErrorInfo(JID_MACRO_PERMISSION, Util::s2ws(JIN_MACRO_PERMISSION));        
        m_dvcOutput->appendResult(errorInfo);

        /*std::cout << JID_MACRO_PERMISSION << std::endl;*/ // error 항목(ID값)        
    }

    return true;
}

bool Checker::CheckCharShapeToCheckList(CharPr* charPr)
{
    ICharShape* charshape = (ICharShape*)m_checkList->getModel(DVCType::CharShape);
    std::map<std::string, int> checkedMapData = charshape->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();

    std::vector<LangSet*> langSets;
    std::vector<BorderInfo*> borderInfos;

    for (iter; iter != checkedMapData.end(); iter++)
    {
        bool isError = false;
        switch (iter->second)
        {
        case JID_CHAR_SHAPE:
            break;
        case JID_CHAR_SHAPE_FONTSIZE:
            if (charshape->getFontSize().min > charPr->height ||
                charshape->getFontSize().max < charPr->height)
                isError = true;            
            break;                        
        case JID_CHAR_SHAPE_LANG:            
            break;
        case JID_CHAR_SHAPE_FONT:
        {
            LPCWSTR fontName = m_owpmlReader->getFontFace(charPr->charPrID, true);
            bool bFind = false;

            std::vector<std::string> fontNames = charshape->getFontNames();
            std::vector<std::string>::iterator iter1 = fontNames.begin();
            for (iter1; iter1 != fontNames.end(); iter1++)
            {
                std::string strName = (*iter1);
                std::wstring strWName = Util::s2ws(strName);
                if (strWName.compare(fontName) == 0) {        // wstring과 string 변환 해 줘야 한다. . json은 string인데 이를 wstring으로 변환해 주는 작업을 해줘야 겠다.
                    bFind = true;
                    break;
                }
            }

            if (!bFind)
                isError = true;

            break;
        }
        case JID_CHAR_SHAPE_RSIZE:        // 상대크기
            if (charshape->getRSize().min > charPr->charPrInfo.relSz ||
                charPr->charPrInfo.relSz > charshape->getRSize().max)
                isError = true;
            break;
        case JID_CHAR_SHAPE_POSITION:
            break;
        case JID_CHAR_SHAPE_RATIO:
            if (charshape->getRatio().min > charPr->charPrInfo.ratio ||
                charPr->charPrInfo.ratio > charshape->getRatio().max)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SPACING:
            if (charshape->getSpacing().min > charPr->charPrInfo.spacing ||
                charPr->charPrInfo.spacing > charshape->getSpacing().max)
                isError = true;
            break;
        case JID_CHAR_SHAPE_BOLD:
            if (charshape->getBold() != charPr->charPrInfo.bold)
                isError = true;
            break;
        case JID_CHAR_SHAPE_ITALIC:
            if (charshape->getItalic() != charPr->charPrInfo.italic)
                isError = true;
            break;
        case JID_CHAR_SHAPE_UNDERLINE:
            if (charshape->getUnderline() != charPr->charPrInfo.underline)
                isError = true;
            break;
        case JID_CHAR_SHAPE_STRIKEOUT:
            if (charshape->getStrikeout() != charPr->charPrInfo.strikeout)
                isError = true;
            break;
        case JID_CHAR_SHAPE_OUTLINE:
            if (charshape->getOoutline() != charPr->charPrInfo.outline)
                isError = true;
            break;
        case JID_CHAR_SHAPE_EMBOSS:
            if (charshape->getEmboss() != charPr->charPrInfo.emboss)
                isError = true;
            break;
        case JID_CHAR_SHAPE_ENGRAVE:
            if (charshape->getEngrave() != charPr->charPrInfo.engrave)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SHADOW:
            if (charshape->getShadow() != charPr->charPrInfo.shadow)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SUPSCRIPT:
            if (charshape->getSupscript() != charPr->charPrInfo.supscript)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SUBSCRIPT:
            if (charshape->getSubscript() != charPr->charPrInfo.subscript)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SHADOWTYPE:        
            if (charshape->getShadowType() != charPr->charPrInfo.shadowType)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SHADOW_X:
            if (charshape->getShadowXDirection() != charPr->charPrInfo.shadowX)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SHADOW_Y:
            if (charshape->getShadowYDirection() != charPr->charPrInfo.shadowY)
                isError = true;
            break;
        case JID_CHAR_SHAPE_SHADOW_COLOR:
            if (charshape->getShadowColor() != charPr->charPrInfo.shadowColor)
                isError = true;
            break;
        case JID_CHAR_SHAPE_UNDERLINE_POSITOION:
            charshape->getUnderlinePos();
            break;
        case JID_CHAR_SHAPE_UNDERLINE_SHAPE:
            charshape->getUnderlineShape();
            break;
        case JID_CHAR_SHAPE_UNDERLINE_COLOR:
            charshape->getUnderlineColor();
            break;
        case JID_CHAR_SHAPE_STRIKEOUT_SHAPE:
            charshape->getStrikeoutShape();
            break;
        case JID_CHAR_SHAPE_STRIKEOUT_COLOR:
            charshape->getStrikeColor();
            break;
        case JID_CHAR_SHAPE_OUTLINETYPE:
            charshape->getOutlineShape();
            break;
        case JID_CHAR_SHAPE_EMPTYSPACE:
            charshape->getEmptyspace();
            break;
        case JID_CHAR_SHAPE_POINT:
            charshape->getPoint();
            break;
        case JID_CHAR_SHAPE_KERNING:
            if (charshape->getKerning() != charPr->useKerning)
                isError = true;
            break;
        case JID_CHAR_SHAPE_BG_BORDER:
        {
            charshape->getBorderInfo(LinePosition::Top);
            charshape->getBorderInfo(LinePosition::Bottom);
            charshape->getBorderInfo(LinePosition::Left);
            charshape->getBorderInfo(LinePosition::Right);

            borderInfos = charshape->getBorderInfos();
            if (borderInfos.size() > 0)
            {
                std::vector<BorderInfo*>::iterator iter = borderInfos.begin();

                for (iter; iter != borderInfos.end(); iter++)
                {
                    BorderInfo* info = (*iter);

                    std::vector<UINT>::iterator checkiter = info->checkedList.begin();

                    switch ((*checkiter))
                    {
                    case JID_CHAR_SHAPE_BG_BORDER_POSITION:
                        (*iter)->position;
                        break;
                    case JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE:
                        (*iter)->lineShape;
                        break;
                    case JID_CHAR_SHAPE_BG_BORDER_SIZE:
                        (*iter)->size;
                        break;
                    case JID_CHAR_SHAPE_BG_BORDER_COLOR:
                        (*iter)->color;
                        break;
                    }
                }
            }
            break;
        }
        case JID_CHAR_SHAPE_BG_COLOR:
            charshape->getBackgroundColor();
            break;
        case JID_CHAR_SHAPE_BG_PATTONCOLOR:
            charshape->getPattonColor();
            break;
        case JID_CHAR_SHAPE_BG_PATTONTYPE:
            charshape->getPattonType();
            break;
        }

        if (isError)
        {
            PushErrorInfo(charPr->charPrID, iter->second, iter->first);
        }
    }

    if (m_errorInfos.size() > 0)
        return false;

    return true;
}

bool Checker::CheckParaShapeToCheckList(ParaPr* paraPr)
{
    IParaShape* paraShape = (IParaShape*)m_checkList->getModel(DVCType::ParaShape);
    std::map<std::string, int> checkedMapData = paraShape->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();

    std::vector<LangSet*> langSets;
    std::vector<BorderInfo*> borderInfos;

    for (iter; iter != checkedMapData.end(); iter++)
    {
        bool isError = false;
        switch (iter->second)
        {
        case JID_PARA_SHAPE:
            break;
        case JID_PARA_SHAPE_HORIZONTAL:
            break;
        case JID_PARA_SHAPE_MARGINLEFT:
            break;
        case JID_PARA_SHAPE_MARGINRIGHT:
            break;
        case JID_PARA_SHAPE_FIRSTLINE:
            break;
        case JID_PARA_SHAPE_INDENT:
            if (paraShape->getIndent().max < paraPr->margin.indent ||
                paraShape->getIndent().min > paraPr->margin.indent)
                isError = true;
            break;
        case JID_PARA_SHAPE_OUTDENT:
            if (paraShape->getOutdent().max < paraPr->margin.indent ||
                paraShape->getOutdent().min > paraPr->margin.indent)
                isError = true;
            break;
        case JID_PARA_SHAPE_LINESPACING:
            if (paraShape->getLinespacing() != paraPr->lineSpacing.type)
                isError = true;
            break;
        case JID_PARA_SHAPE_LINESPACINGVALUE:
            if (paraShape->getLinespacingvalue().max < paraPr->lineSpacing.value ||
                paraShape->getLinespacingvalue().min > paraPr->lineSpacing.value)
                isError = true;
            break;
        case JID_PARA_SHAPE_SPACINGPARAUP:
            if (paraShape->getSpacingParaup().max < paraPr->margin.prev ||
                paraShape->getSpacingParaup().min > paraPr->margin.prev)
                isError = true;
            break;
        case JID_PARA_SHAPE_SPACINGPARABOTTOM:
            if (paraShape->getSpacingParabottom().max < paraPr->margin.next ||
                paraShape->getSpacingParabottom().min > paraPr->margin.next)
                isError = true;
            break;
        case JID_PARA_SHAPE_SPACINGGRIDPAPER:
            break;
        case JID_PARA_SHAPE_LINEBREAKKOREAN:
            break;
        case JID_PARA_SHAPE_LINEBREAKENGLISH:
            break;
        case JID_PARA_SHAPE_LINEBREAKCONDENSE:
            break;
        case JID_PARA_SHAPE_PARATYPE:
            break;
        case JID_PARA_SHAPE_PARATYPEVALUE:
            break;
        case JID_PARA_SHAPE_WIDOWORPHAN:
            break;
        case JID_PARA_SHAPE_KEEPWITHNEXT:
            break;
        case JID_PARA_SHAPE_KEEPLINESTOGETHER:
            break;
        case JID_PARA_SHAPE_PAGEBREAKBEFORE:
            break;
        case JID_PARA_SHAPE_FONTLINEHEIGHT:
            break;
        case JID_PARA_SHAPE_LINEWRAP:
            break;
        case JID_PARA_SHAPE_AUTOSPACEEASIANENG:
            break;
        case JID_PARA_SHAPE_AUTOSPACEEASIANNUM:
            break;
        case JID_PARA_SHAPE_VERTICALALIGN:
            break;
        case JID_PARA_SHAPE_TABTYPES:
            break;
        case JID_PARA_SHAPE_TABTYPE:
            break;
        case JID_PARA_SHAPE_TABSHAPE:
            break;
        case JID_PARA_SHAPE_TABPOSITION:
            break;
        case JID_PARA_SHAPE_AUTOTABINDENT:
            break;
        case JID_PARA_SHAPE_AUTOTABPARARIGHTEND:
            break;
        case JID_PARA_SHAPE_BASETABSPACE:
            break;
        case JID_PARA_SHAPE_BORDER:
            break;
        case JID_PARA_SHAPE_BORDERPOSITION:
            break;
        case JID_PARA_SHAPE_BORDERTYPE:
            break;
        case JID_PARA_SHAPE_BORDERSIZE:
            break;
        case JID_PARA_SHAPE_BORDERCOLOR:
            break;
        case JID_PARA_SHAPE_BGCOLOR:
            break;
        case JID_PARA_SHAPE_BGPATTONCOLOR:
            break;
        case JID_PARA_SHAPE_BGPATTONTYPE:
            break;
        case JID_PARA_SHAPE_SPACINGLEFT:
            break;
        case JID_PARA_SHAPE_SPACINGRIGHT:
            break;
        case JID_PARA_SHAPE_SPACINGTOP:
            break;
        case JID_PARA_SHAPE_SPACINGBOTTOM:
            break;
        case JID_PARA_SHAPE_SPACINGIGNORE:
            break;
        }

        if (isError)
        {
            PushErrorInfo(paraPr->paraPrID, iter->second, iter->first);
        }
    }
    if (m_errorInfos.size() > 0)
        return false;

    return true;
}

bool Checker::CheckTableToCheckList(RTable* table, RBorderFills* borderFills)
{
    ITable* iTable = (ITable*)m_checkList->getModel(DVCType::Table);
    std::map<std::string, int> checkedMapData = iTable->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();

    //std::vector<LangSet*> langSets;
    //std::vector<BorderInfo*> borderInfos;    

    for (iter; iter != checkedMapData.end(); iter++)
    {
        bool isError = false;
        UINT elementNo = iter->second;
        switch (elementNo)
        {
        case JID_TABLE_SIZEWIDTH:
        {            
            if (iTable->getSizeWidth().max < table->getWidth() ||
                iTable->getSizeWidth().min > table->getWidth())
                isError = true;
            break;
        }
        case JID_TABLE_SIZEHEIGHT:
        {
            if (iTable->getSizeHeight().max < table->getHeight() ||
                iTable->getSizeHeight().min > table->getHeight())
                isError = true;
            break;
        }
        case JID_TABLE_SIZEFIXED:
        {
            if (iTable->getSizeFixed() != table->getLock())
                isError = true;
            break;
        }
        case JID_TABLE_TREATASCHAR:
        {
            if (iTable->getTreatAsChar() && !table->getTreatAsChar())
                isError = true;
            break;
        }
        case JID_TABLE_POS:
        {
            if (iTable->getPosition() != table->getTextWrap())
                isError = true;
            break;
        }
        case JID_TABLE_TEXTPOS:
        {
            if (iTable->getTextPos() != table->getTextFlow())
                isError = true;
            break;
        }
        case JID_TABLE_HTYPE:
            break;
        case JID_TABLE_HDIRECTION:
            break;
        case JID_TABLE_HVALUE:
            break;
        case JID_TABLE_VTYPE:
            break;
        case JID_TABLE_VDIRECTION:
            break;
        case JID_TABLE_VVALUE:
            break;
        case JID_TABLE_SOFLOWWITHTEXT:
            break;
        case JID_TABLE_SOALLOWOVERLAP:
            break;
        case JID_TABLE_SOHOLDANCHOROBJ:
            break;
        case JID_TABLE_PARALLEL:
            break;
        case JID_TABLE_ROTATION:
            break;
        case JID_TABLE_GRADIENT_H:
            break;
        case JID_TABLE_GRADIENT_V:
            break;
        case JID_TABLE_NUMVERTYPE:
            break;
        case JID_TABLE_OBJPROTECT:
            break;
        case JID_TABLE_MARGIN_LEFT:
            break;
        case JID_TABLE_MARGIN_RIGHT:
            break;
        case JID_TABLE_MARGIN_TOP:
            break;
        case JID_TABLE_MARGIN_BOTTOM:
            break;
        case JID_TABLE_CAPTION_POSITION:
            break;
        case JID_TABLE_CAPTION_SIZE:
            break;
        case JID_TABLE_CAPTION_SPACING:
            break;
        case JID_TABLE_CAPTION_SOCAPFULLSIZE:
            break;
        case JID_TABLE_CAPTION_LINEWRAP:
            break;
        case JID_TABLE_BORDER:
        {            
            bool bResult = true;
            std::vector<BorderInfo*> borderInfos = iTable->getBorderInfos();
            
            std::vector<RTableRow*> rows = table->getTableRows();
            std::vector<RTableRow*>::iterator rowIter = rows.begin();
            for (rowIter; rowIter != rows.end(); rowIter++)
            {
                std::vector<RTableColumn*> cols = (*rowIter)->getColumns();
                std::vector<RTableColumn*>::iterator colIter = cols.begin();

                for (colIter; colIter != cols.end(); colIter++)
                {
                    
                    BorderFill* borderFill = borderFills->getBorderFill((*colIter)->getBorderFillIDRef());

                    bResult = CheckFromBorderInfo(borderFill, borderInfos, table->getTableID(), (*colIter)->getCol(), (*colIter)->getRow());

                    if (!bResult && !m_checkTableDetail)    // 셀단위로 체크하지 않고 테이블단위로 체크 하는 경우에는 오류가 있을때 바로 종료.
                        break;
                }

                if (!bResult && !m_checkTableDetail)    // 셀단위로 체크하지 않고 테이블단위로 체크 하는 경우에는 오류가 있을때 바로 종료.
                    break;
            }
            break;            
        }
        
        case JID_TABLE_BORDER_CELLSPACING:
        {
            break;
        }
        case JID_TABLE_BGFILL_TYPE:
        {
            break;
        }
        case JID_TABLE_BGFILL_FACECOLOR:
        {
            break;
        }
        case JID_TABLE_BGFILL_PATTONCOLOR:
        {
            break;
        }
        case JID_TABLE_BGFILL_PATTONTYPE:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_STARTCOLOR:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_ENDCOLOR:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_TYPE:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_WIDTHCENTER:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_HEIGHTCENTER:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_GRADATIONANGLE:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_BLURLEVEL:
        {
            break;
        }
        case JID_TABLE_BGGRADATION_BLURCENTER:
        {
            break;
        }
        case JID_TABLE_PICTURE_FILE:
        {
            break;
        }
        case JID_TABLE_PICTURE_INCLUDE:
        {
            break;
        }
        case JID_TABLE_PICTUREFILL_TYPE:
        {
            break;
        }
        case JID_TABLE_PICTUREFILL_VALUE:
        {
            break;
        }
        case JID_TABLE_EFFECT_TYPE:
        {
            break;
        }
        case JID_TABLE_EFFECT_VALUE:
        {
            break;
        }
        case JID_TABLE_WATERMARK:
        {
            break;
        }
        case JID_TABLE_TABLE_IN_TABLE:
        {
            if (!iTable->getTableInTable() && table->getTableInTable())    // iTable이 false이면 테이믈 안에 테이블이 있으면 안됨을 의미함.
                isError = true;
            
            break;
        }
        default:
            break;
        }

        if (isError)
        {            
            PushErrorInfo(table->getTableID(), elementNo, iter->first);
        }
    }
    if (m_errorInfos.size() > 0)
        return false;

    return true;

}

bool Checker::CheckFromBorderInfo(BorderFill* borderFill, std::vector<BorderInfo*> borderInfos, UINT tableID, UINT col, UINT row)
{    
    bool isError = false;

    if (borderInfos.size() > 0)
    {
        std::vector<BorderInfo*>::iterator iter = borderInfos.begin();

        for (iter; iter != borderInfos.end(); iter++)
        {
            BorderInfo* info = (*iter);

            if (isError)    // 해당 셀의 border의 속성 중 하나라도 error가 있으면 나머지는 체크하지 않고 하나만 기록한다.
                break;

            if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_POSITION))
            {
                if (info->position == LinePosition::Top)
                {
                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_TYPE))
                    {
                        if (info->lineShape != borderFill->topType)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_SIZE))
                    {
                        if (info->size != borderFill->topWidth)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_COLOR))
                    {
                        if (info->color != borderFill->topColor)
                            isError = true;
                    }
                }
                else if (info->position == LinePosition::Bottom)
                {
                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_TYPE))
                    {
                        if (info->lineShape != borderFill->bottomType)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_SIZE))
                    {
                        if (info->size != borderFill->bottomWidth)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_COLOR))
                    {
                        if (info->color != borderFill->bottomColor)
                            isError = true;
                    }
                }
                else if (info->position == LinePosition::Left)
                {
                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_TYPE))
                    {
                        if (info->lineShape != borderFill->leftType)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_SIZE))
                    {
                        if (info->size != borderFill->leftWidth)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_COLOR))
                    {
                        if (info->color != borderFill->leftColor)
                            isError = true;
                    }
                }
                else if (info->position == LinePosition::Right)
                {
                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_TYPE))
                    {
                        if (info->lineShape != borderFill->rightType)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_SIZE))
                    {
                        if (info->size != borderFill->rightWidth)
                            isError = true;
                    }

                    if (Util::findInVector(info->checkedList, JID_TABLE_BORDER_COLOR))
                    {
                        if (info->color != borderFill->rightColor)
                            isError = true;
                    }
                }
            }

            if (isError)
            {                
                PushErrorInfo(tableID, JID_TABLE_BORDER, JIN_TABLE_BORDER, col, row);
            }
        }
    }

    return !isError;
}

bool Checker::CheckSpacialCharacterToCheckList()
{
    ISpecialCharacter* spacial = (ISpecialCharacter*)m_checkList->getModel(DVCType::SpecialCharacter);
        
    std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();

    unsigned int minValue = spacial->getMinimum();
    unsigned int maxValue = spacial->getMaximum();

    for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
    {
        std::wstring text = (*runTypeIter)->text;

        for (int i = 0; i < text.size(); i++)
        {
            if (minValue > text[i] || text[i] > maxValue)
            {                
                PushErrorInfo((*runTypeIter)->charPrIDRef, JID_SPECIALCHARACTER, JIN_SPECIALCHARACTER);
            }
        }
    }

    return true;
}

bool Checker::CheckOutlineShapeToCheckList(ParaType type)
{    
    std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();

    RParaShape* rParaShape = m_owpmlReader->getParaShape();
    

    for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
    {
        ParaPr* rParaPr = rParaShape->getParaPropertyForID((*runTypeIter)->paraPrIDRef);
        ROutlineShape* outline = m_owpmlReader->getOutlineShape();
        UINT id = 0;

        if (!((type == ParaType::ParaOutline && rParaPr->headingType == ParaType::ParaOutline) ||
            (type == ParaType::ParaNumber && rParaPr->headingType == ParaType::ParaNumber)))
            continue;

        if (type == ParaType::ParaOutline)
            id = (*runTypeIter)->outlineShapeIDRef;
        else
            id = rParaPr->headingIdRef;

        RNumbering* numbering = outline->getNumbering(id);
        ParaHead* paraHead = numbering->getParaHeadByLevel(rParaPr->headingLevel);
        if (type == ParaType::ParaOutline)
            CheckOutlineParaHeadToCheckList(paraHead, (*runTypeIter)->paraPrIDRef);
        else
            CheckNumberParaHeadToCheckList(paraHead, (*runTypeIter)->paraPrIDRef);        
    }    

    return true;
}

bool Checker::CheckOutlineParaHeadToCheckList(ParaHead* paraHead, UINT paraPrIDRef)
{
    IOutlineShape* outlineShape = (IOutlineShape*)m_checkList->getModel(DVCType::OutlineShape);

    std::vector<LevelType*> types = outlineShape->getLevelTypes();
    std::vector<LevelType*>::iterator Leveliter = types.begin();

    std::map<std::string, int> checkedMapData = outlineShape->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();


    LevelType* levelType = getLevelType(types, paraHead->level);
    std::wstring numType = ReaderUtil::s2ws(levelType->numberType);
    if (!numType.compare(paraHead->numFormat.c_str()))
    {        
        PushErrorInfo(paraPrIDRef, JID_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE, JIN_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE);
    }

    if (levelType->numShapeType != paraHead->numType)
    {        
        PushErrorInfo(paraPrIDRef, JID_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE, JIN_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE);
    }

    return true;
}

bool Checker::CheckNumberParaHeadToCheckList(ParaHead* paraHead, UINT paraPrIDRef)
{
    IParaNumBullet* paraNumBullet = (IParaNumBullet*)m_checkList->getModel(DVCType::ParaNumBullet);

    std::vector<LevelType*> types = paraNumBullet->getLevelTypes();
    std::vector<LevelType*>::iterator Leveliter = types.begin();

    std::map<std::string, int> checkedMapData = paraNumBullet->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();
    
    LevelType* levelType = getLevelType(types, paraHead->level);
    std::wstring numType = ReaderUtil::s2ws(levelType->numberType);
    if (!numType.compare(paraHead->numFormat.c_str()))
    {
        PushErrorInfo(paraPrIDRef, JID_PARANUMBULLET_LEVELTYPE_NUMBERTYPE, JIN_PARANUMBULLET_LEVELTYPE_NUMBERTYPE);
    }

    if (levelType->numShapeType != paraHead->numType)
    {
        PushErrorInfo(paraPrIDRef, JID_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE, JIN_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE);
    }

    return true;
}

LevelType* Checker::getLevelType(std::vector<LevelType*> levelTypes, UINT level)
{
    std::vector<LevelType*>::iterator iter = levelTypes.begin();

    for (iter; iter != levelTypes.end(); iter++)
    {
        if ((*iter)->level == level)
            return (*iter);
    }

    return NULL;
}

bool Checker::CheckBulletToCheckList()
{
    IBullet* iBullet = (IBullet*)m_checkList->getModel(DVCType::Bullet);
        
    std::vector<RunTypeInfo*>::iterator runTypeIter = m_runTypeInfo.begin();

    RParaShape* rParaShape = m_owpmlReader->getParaShape();


    for (runTypeIter; runTypeIter != m_runTypeInfo.end(); runTypeIter++)
    {
        ParaPr* rParaPr = rParaShape->getParaPropertyForID((*runTypeIter)->paraPrIDRef);

        if (rParaPr->headingType != ParaType::ParaBullet)
            continue;

        if (rParaPr->headingType == ParaType::ParaBullet)
        {
            RBullets* bullets = m_owpmlReader->getBullets();
            BulletHead* bullet = bullets->getBullet(rParaPr->headingIdRef);

            std::string bulletshapes = iBullet->getBulletShapes();
            std::wstring shapes = ReaderUtil::s2ws(bulletshapes);

            size_t nCnt = shapes.size();
            bool isFind = false;

            for (int i = 0; i < nCnt; i++)
            {
                if (shapes[i] == bullet->bulletChar[0])    // 허용 되는것만 찾기
                {
                    isFind = true;
                    break;
                }
            }

            if (!isFind)
            {
                PushErrorInfo((*runTypeIter)->paraPrIDRef, JID_BULLET_SHAPES, JIN_BULLET_SHAPES);
            }
        }
    }

    return true;
}


bool Checker::CheckHyperlinkToRunInfo()
{
    bool useHyperlink = true;

    IHyperlink* hyperlink = (IHyperlink*)m_checkList->getModel(DVCType::Hyperlink);

    std::map<std::string, int> checkedMapData = hyperlink->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();


    for (iter; iter != checkedMapData.end(); iter++)
    {
        bool isError = false;
        switch (iter->second)
        {
        case JID_HYPERLINK_PERMISSION:
            isError = true;
            useHyperlink = hyperlink->getPermission();
            break;        
        }
    }

    return useHyperlink;
}

bool Checker::CheckStyleToRunInfo()
{
    bool useStyle = true;

    IStyle* style = (IStyle*)m_checkList->getModel(DVCType::Style);

    std::map<std::string, int> checkedMapData = style->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();

    for (iter; iter != checkedMapData.end(); iter++)
    {
        bool isError = false;
        switch (iter->second)
        {
        case JID_STYLE_PERMISSION:
            isError = true;
            useStyle = style->getPermission();
            break;
        }
    }

    return useStyle;
}

bool Checker::CheckMacroToCheckLIst()
{    
    bool isError = false;
    bool useMacro = true;

    bool haveMacro = m_owpmlReader->haveMacroInDocument();

    IMacro* macro = (IMacro*)m_checkList->getModel(DVCType::Macro);

    std::map<std::string, int> checkedMapData = macro->getMapData();
    std::map<std::string, int>::iterator iter = checkedMapData.begin();

    if (checkedMapData.size() == 0)
        return true;

    for (iter; iter != checkedMapData.end(); iter++)
    {
        switch (iter->second)
        {
        case JID_MACRO_PERMISSION:
            isError = true;
            useMacro = macro->getPermission();
            break;
        }
    }

    if (!useMacro && haveMacro)
        return false;    // error
}

void Checker::PushErrorInfo(UINT id, UINT elementID, std::string elementString, UINT col, UINT row)
{
    ErrorInfo* errorInfo = new ErrorInfo;
    errorInfo->IDRef = id;
    errorInfo->elementID = elementID;
    errorInfo->elementString = elementString;

    errorInfo->row = row;
    errorInfo->col = col;

    m_errorInfos.push_back(errorInfo);
}

void Checker::ClearErrorInfos()
{
    if (m_errorInfos.size() > 0)
    {
        std::vector<ErrorInfo*>::iterator iter = m_errorInfos.begin();

        for (iter; iter != m_errorInfos.end(); iter++)
        {
            if ((*iter))
                delete *iter;
            *iter = NULL;
        }
        m_errorInfos.clear();
    }
}

RunTypeInfo* Checker::CloneRunInfo(RunTypeInfo* info, ErrorInfo* errInfo)
{
    RunTypeInfo* newInfo = new RunTypeInfo;
    
    *newInfo = *info;
    if (errInfo)
    {
        newInfo->errorCode = errInfo->elementID;
        newInfo->errorString = Util::s2ws(errInfo->elementString);
    }

    return newInfo;
}