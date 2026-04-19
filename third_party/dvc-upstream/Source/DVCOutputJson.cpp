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
#include "DVCOutputJson.h"
#include "../DVCDefine.h"
#include "../Util.h"

#include <fstream>

DVCOutputJson::DVCOutputJson()
{
    m_errorInfos.clear();
    m_json.clear();
}

DVCOutputJson::~DVCOutputJson()
{
    m_json.clear();

    if (m_errorInfos.size() > 0)
    {
        std::vector<IDVCErrInfo*>::iterator iter = m_errorInfos.begin();

        for (iter; iter != m_errorInfos.end(); iter++)
        {
            if ((*iter))
                delete *iter;
            *iter = NULL;
        }
        m_errorInfos.clear();
    }
}


OutputType DVCOutputJson::getType()
{
    return OutputType::JSON;
}

void DVCOutputJson::appendResult(IDVCErrInfo* result)
{
    if (result == NULL) {
        return;
    }

    m_errorInfos.push_back(result);
}

void DVCOutputJson::writeToFile(const DVC_STRING& filename, DVCOutputOption option)
{
    m_outputFile = filename;

    //Json::Value json = makeJsonBuffer(option);
    if (m_json.empty())
        m_json = makeJsonBuffer(option);

    Json::StreamWriterBuilder builder;
    builder.settings_["emitUTF8"] = true;
    std::unique_ptr<Json::StreamWriter> writer(builder.newStreamWriter());

    std::string path = Util::ws2s(m_outputFile);
    std::ofstream writeOfStream(path.c_str());
    writer->write(m_json, &writeOfStream);

    writeOfStream.close();

}

void DVCOutputJson::writeToConsole(DVCOutputOption option)
{
    //Json::Value json = makeJsonBuffer(option);
    if (m_json.empty())
        m_json = makeJsonBuffer(option);

    std::string jsonString = m_json.toStyledString();

#ifdef OS_LINUX
    std::cout << jsonString.c_str() << std::endl;
#else
    HANDLE output = GetStdHandle(STD_OUTPUT_HANDLE);
    DWORD dwWritten = 0;
    
    //WriteConsole(output, jsonString.c_str(), jsonString.length(), &dwWritten, NULL);
    std::cout << jsonString.c_str() << std::endl;
#endif
}

Json::Value DVCOutputJson::makeJsonBuffer(DVCOutputOption option)
{
    std::vector<IDVCErrInfo*>::iterator iter = m_errorInfos.begin();

    Json::Value    json;
    //Json::Value value;
    for (iter; iter != m_errorInfos.end(); iter++)
    {
        Json::Value value;
        if ((*iter) != NULL)
        {
            value[DVCString::CharIDRef] = (*iter)->getCharPrIDRef();
            value[DVCString::ParaPrIDRef] = (*iter)->getParaPrIDRef();

            setlocale(LC_ALL, "");
            std::wstring text = (*iter)->getText();        
            std::string text1;
            std::string text2;

            int len = text.length();
            char* mbsName = NULL;
                                    
            size_t sizeVal = 0;
            if (len > 0)
            {
                mbsName = new char[len*2];
                wcstombs_s(&sizeVal, mbsName, len * 2 + 1, text.c_str(), _TRUNCATE);
                text1.append(mbsName);
            }

            value[DVCString::errorText] = (text.size() > 0) ? text1.c_str() : "";
            value[DVCString::PageNo] = (*iter)->getPageNo();
            value[DVCString::LineNo] = (*iter)->getLineNo();
            value[DVCString::ErrorCode] = (*iter)->getErrorCode();

            if (option == DVCOutputOption::Table ||
                option == DVCOutputOption::TableDetail ||
                option == DVCOutputOption::AllOption)
            {
                value[DVCString::TableID] = (*iter)->getTableID();
                value[DVCString::IsInTable] = (*iter)->getIsInTable();
                value[DVCString::IsInTableInTable] = (*iter)->getIsInTableInTable();
                value[DVCString::TableRow] = (*iter)->getTableRow();
                value[DVCString::TableCol] = (*iter)->getTableCol();
            }
            else if (option == DVCOutputOption::Style ||
                option == DVCOutputOption::AllOption)
            {
                value[DVCString::UseStyle] = (*iter)->getUseStyle();
            }
            else if (option == DVCOutputOption::Shape ||
                option == DVCOutputOption::AllOption)
            {
                value[DVCString::IsInShape] = (*iter)->getIsInShape();
            }
            else if (option == DVCOutputOption::Hyperlink ||
                option == DVCOutputOption::AllOption)
            {
                value[DVCString::UseHyperlink] = (*iter)->getUseHyperlink();
            }

            if (text.empty())
            {
                if (option != DVCOutputOption::Table ||
                    option != DVCOutputOption::TableDetail ||
                    option != DVCOutputOption::AllOption)
                    value.clear();
            }
        }
        if (!value.empty())
        {
            json.append(value);
        }        
    }

    return json;
}

void DVCOutputJson::doSortErrorInfos()
{
    std::vector<IDVCErrInfo*>::iterator iter = m_errorInfos.begin();
    for (iter; iter != m_errorInfos.end(); iter++)
    {
        std::vector<IDVCErrInfo*>::iterator iter2 = iter+1;
        for (iter2; iter2 != m_errorInfos.end(); iter2++)
        {
            if ((*iter)->getCharPrIDRef() == (*iter2)->getCharPrIDRef() &&
                (*iter)->getParaPrIDRef() == (*iter2)->getCharPrIDRef() &&
                (*iter)->getText() == (*iter2)->getText() &&
                (*iter)->getPageNo() == (*iter2)->getPageNo() &&
                (*iter)->getLineNo() == (*iter2)->getLineNo() &&
                (*iter)->getErrorCode() == (*iter2)->getErrorCode())
            {
                if ((*iter2))
                {
                    delete *iter2;
                    *iter2 = NULL;
                }
            }
        }
    }
}
