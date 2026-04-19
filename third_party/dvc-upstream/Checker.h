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
#include "Source\CheckList.h"
#include "Source\OWPMLReader.h"

#include "Source\OWPMLReader.h"

#include "ExportInterface.h"

//class OutputModel;
class DVCModel;
class HwpxModel;

using namespace OWPMLReaderModule;

typedef struct ErrorInfo
{
    UINT            IDRef;
    UINT            elementID;
    UINT            row;
    UINT            col;
    std::string        elementString;
};

class Checker
{
public:
    Checker(std::wstring checklistFile, std::wstring sourceFile);
    ~Checker();
    
public:
    // outputmodel 을 만들때, Json::Value m_Json; 를 사용해보자. 이렇게 값을 넣게 되면 자동으로 json을 만들수 있을듯 하다.
    // 이는 한글코드에서 위와같이 검색해보면 사용법은 나온다.HwpJson.cpp/h
    bool    Initialize();
    void    SetOutput(DVC::IDVCOutput* output);
       
    bool    CheckCharShape();
    bool    CheckParaShape();
    bool    CheckTable(bool checkDetail);
    bool    CheckSpacialCharacter();

    bool    CheckOutlineShape();
    bool    CheckBullet();
    bool    CheckParaNumBullet();

    bool    CheckHyperlink();
    bool    CheckStyle();

    bool    CheckMacro();

    bool    PrintCheckedErrorList();

private:
    bool    CheckCharShapeToCheckList(CharPr* charPr);
    bool    CheckParaShapeToCheckList(ParaPr* paraPr);
    bool    CheckTableToCheckList(RTable* table, RBorderFills* borderFills);
    bool    CheckFromBorderInfo(BorderFill* borderFill, std::vector<BorderInfo*> borderInfos, UINT tableID, UINT col, UINT row);
    bool    CheckSpacialCharacterToCheckList();

    bool    CheckOutlineShapeToCheckList(ParaType type);
    bool    CheckOutlineParaHeadToCheckList(ParaHead* paraHead, UINT paraPrIDRef);
    bool    CheckNumberParaHeadToCheckList(ParaHead* paraHead, UINT paraPrIDRef);
    bool    CheckBulletToCheckList();    

    bool    CheckHyperlinkToRunInfo();
    bool    CheckStyleToRunInfo();

    bool    CheckMacroToCheckLIst();

    LevelType* getLevelType(std::vector<LevelType*> levelTypes, UINT level);

    //void    PushErrorInfo(UINT id, UINT elementID);
    void    PushErrorInfo(UINT id, UINT elementID, std::string elementString, UINT col = 0, UINT row = 0);
    void    ClearErrorInfos();

    RunTypeInfo* CloneRunInfo(RunTypeInfo* info, ErrorInfo* errInfo = NULL);


    CheckList*            m_checkList;
    OWPMLReader*        m_owpmlReader;

    DVC::IDVCOutput*    m_dvcOutput;

    std::wstring        m_checkListFile;
    std::wstring        m_sourceFile;

    bool                m_checkTableDetail;        // true : 테이블을 셀 단위로 체크        false : 테이블에서 오류가 있으면 바로 중단하고 다음 테이블로 넘어감.

    std::vector<RunTypeInfo*> m_runTypeInfo;
    std::vector<RunTypeInfo*> m_checkedErrorListInfo;    // m_runTypeInfo와 공존하기 때문에 이 변수는 따로 delete 하지 않아도 됨.
    std::vector<ErrorInfo*>    m_errorInfos;
};

