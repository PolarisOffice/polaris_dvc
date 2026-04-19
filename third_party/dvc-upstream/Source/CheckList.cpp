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
#include "CheckList.h"


static std::map<std::string, int> checkListMapData = {
    {JIN_CHAR_SHAPE,                JID_CHAR_SHAPE},
    {JIN_PARA_SHAPE,                JID_PARA_SHAPE},
    {JIN_TABLE,                        JID_TABLE},
    {JIN_SPECIALCHARACTER,            JID_SPECIALCHARACTER},
    {JIN_OUTLINESHAPE,                JID_OUTLINESHAPE},
    {JIN_BULLET,                    JID_BULLET},
    {JIN_PARANUMBULLET,                JID_PARANUMBULLET},
    {JIN_STYLE,                        JID_STYLE},
    {JIN_PAGE,                        JID_PAGE},
    {JIN_DOCSUMMARYINFO,            JID_DOCSUMMARYINFO},
    {JIN_FOOTNOTE,                    JID_FOOTNOTE},
    {JIN_ENDNOTE,                    JID_ENDNOTE},
    {JIN_MEMO,                        JID_MEMO},
    {JIN_CHART,                        JID_CHART},
    {JIN_WORDART,                    JID_WORDART},
    {JIN_FORMULA,                    JID_FORMULA},
    {JIN_OLE,                        JID_OLE},
    {JIN_FORMOBJECT,                JID_FORMOBJECT},
    {JIN_BOOKMARK,                    JID_BOOKMARK},
    {JIN_HYPERLINK,                    JID_HYPERLINK},
    {JIN_MACRO,                        JID_MACRO}
};

CheckList::CheckList(std::string file)
{
    m_file = file;
}

CheckList::~CheckList()
{
    if (m_objList.size() > 0)
    {
        std::vector<ICheckListObj*>::iterator iter = m_objList.begin();

        for (iter; iter != m_objList.end(); iter++)
        {
            if ((*iter))
                delete *iter;
            *iter = NULL;
        }
        m_objList.clear();
    }
}

bool CheckList::parsing() {
    /*
    1. json을 이곳에서 parsing 한다.
    2. 각 element별로 string을 가지고 와서 해당 class에 넘겨주자. 이때 file로만 넘겨주도록 되어 있으니 이 부분은 추가 구현하자.
    3. 2번과 같이 하면 있는 element만 가지고 오게되고, 있는것만 objList에 push 하니까 나중에 따로 검사를 할 필요가 없다.
    */

    bool bResult = false;

    Json::Reader reader;
    Json::Value root;
    
    FILE *fp = NULL;

    fopen_s(&fp, m_file.c_str(), "rb");
    char* buffer = NULL;
    size_t size = 0;

    if (!fp)
        return false;

    fseek(fp, 0, SEEK_END);
    size = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    buffer = new char[size + 1];
    memset(buffer, 0, size);
    fread(buffer, size, 1, fp);
    fclose(fp);

    if (reader.parse(buffer, root))
    {
        std::map<std::string, int>::iterator mapIter;
        std::vector<std::string> members = root.getMemberNames();

        if (members.size() <= 0)
            return false;

        for (int i = 0; i < members.size(); i++)
        {
            int nID = -1;

            mapIter = checkListMapData.find(members[i]);            

            switch (mapIter->second)
            {
            case JID_CHAR_SHAPE:
            {                
                CheckListModule::CCharShape* charshape = new CheckListModule::CCharShape(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(charshape);
                break;
            }
            case JID_PARA_SHAPE:
            {
                CheckListModule::CParaShape* parashape = new CheckListModule::CParaShape(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(parashape);
                break;
            }
            case JID_TABLE:
            {
                CheckListModule::CTable* table = new CheckListModule::CTable(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(table);
                break;
            }
            case JID_SPECIALCHARACTER:
            {
                CheckListModule::CSpecialCharacter* specialchar = new CheckListModule::CSpecialCharacter(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(specialchar);
                break;
            }
            case JID_OUTLINESHAPE:
            {
                CheckListModule::COutlineShape* outlineshape = new CheckListModule::COutlineShape(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(outlineshape);
                break;
            }
            case JID_BULLET:
            {
                CheckListModule::CBullet* bullet = new CheckListModule::CBullet(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(bullet);
                break;
            }
            case JID_PARANUMBULLET:
            {
                CheckListModule::CParaNumBullet* paranumbullet = new CheckListModule::CParaNumBullet(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(paranumbullet);
                break;
            }
            case JID_STYLE:
            {
                CheckListModule::CStyle* style = new CheckListModule::CStyle(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(style);
                break;
            }
            case JID_PAGE:
                break;
            case JID_DOCSUMMARYINFO:
                break;
            case JID_FOOTNOTE:
                break;
            case JID_ENDNOTE:
                break;
            case JID_MEMO:
                break;
            case JID_CHART:
                break;
            case JID_WORDART:
                break;
            case JID_FORMULA:
                break;
            case JID_OLE:
                break;
            case JID_FORMOBJECT:
                break;
            case JID_BOOKMARK:
                break;
            case JID_HYPERLINK:
            {
                CheckListModule::CHyperlink* hyperlink = new CheckListModule::CHyperlink(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(hyperlink);
                break;
            }
            case JID_MACRO:
                CheckListModule::CMacro* macro = new CheckListModule::CMacro(root[mapIter->first].toStyledString().c_str());
                m_objList.push_back(macro);
                break;            
            }
        }        
    }    

    delete buffer;

    return true;
}

ICheckListObj* CheckList::getModel(DVCType modelType) 
{
    for (ICheckListObj* obj : m_objList) {
        if (obj->getType() == modelType)
            return obj;
    }

    return NULL;
}


