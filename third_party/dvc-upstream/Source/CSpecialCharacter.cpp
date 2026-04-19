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
#include "CSpecialCharacter.h"


static std::map<std::string, int> specialCharacterMapData = {
    {JIN_SPECIALCHARACTER,                            JID_SPECIALCHARACTER},
    {JIN_SPECIALCHARACTER_MINIMUM,                    JID_SPECIALCHARACTER_MINIMUM},
    {JIN_SPECIALCHARACTER_MAXIMUM,                    JID_SPECIALCHARACTER_MAXIMUM}
};

namespace CheckListModule {

    CSpecialCharacter::CSpecialCharacter(std::string dvcFile)
    {
        m_dvcFile = dvcFile;
        parsingDVCFile(m_dvcFile);
    }

    CSpecialCharacter::CSpecialCharacter(LPCSTR document)
    {
        parsingDVCDocument(document);
    }

    CSpecialCharacter::~CSpecialCharacter()
    {

    }

    UINT CSpecialCharacter::getMinimum()
    {
        return m_minimum;
    }

    UINT CSpecialCharacter::getMaximum()
    {
        return m_maximum;
    }

    bool CSpecialCharacter::parsingDVCFile(std::string dvcFile)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;
        std::string document;

        if (dvcFile.length() <= 0)
            dvcFile = m_dvcFile;

        if (reader.parse(document, root))
        {
            Json::Value value = root[JIN_SPECIALCHARACTER];

            bResult = parsingElement(value);
        }

        return bResult;
    }

    bool CSpecialCharacter::parsingDVCDocument(LPCSTR document)
    {
        bool bResult = false;

        Json::Reader reader;
        Json::Value root;        

        if (reader.parse(document, root))
        {
            bResult = parsingElement(root);
        }

        return bResult;
    }

    bool CSpecialCharacter::parsingElement(Json::Value root, void* Obj)
    {
        std::map<std::string, int>::iterator mapIter;
        Json::ValueIterator it = root.begin();

        bool bFind = false;

        while (it != root.end())
        {
            int nValue = 0;
            bool bValue = false;
            std::string strValue = "";
            Json::Value jsonValue;

            bFind = true;

            if (root[it.name().c_str()].isInt())
                nValue = root[it.name().c_str()].asInt();
            else if (root[it.name().c_str()].isBool())
                bValue = root[it.name().c_str()].asBool();
            else if (root[it.name().c_str()].isString())
                strValue = root[it.name().c_str()].asString();
            else if (root[it.name().c_str()].isObject())
                jsonValue = root[it.name().c_str()];

            mapIter = specialCharacterMapData.find(it.name().c_str());    // 또는 it.key()로 해보자.

            switch (mapIter->second)
            {
            case JID_SPECIALCHARACTER_MINIMUM:
                m_minimum = nValue;
                break;
            case JID_SPECIALCHARACTER_MAXIMUM:
                m_maximum = nValue;
                break;            
            default:
                break;
            }

            if (bFind)
            {
                m_checkedList.push_back(mapIter->second);
                m_mapCheckedList.insert(std::map<std::string, int>::value_type(mapIter->first, mapIter->second));
            }

            it++;
        }

        return true;
    }
}