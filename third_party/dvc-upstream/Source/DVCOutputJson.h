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

#include "ExportInterface.h"
#include "json.h"

using namespace DVC;

class DVCOutputJson : public IDVCOutput
{
public:
    DVCOutputJson();
    virtual ~DVCOutputJson();

public:
    OutputType getType();
    void appendResult(IDVCErrInfo* result);
    void writeToFile(const DVC_STRING& filename, DVCOutputOption option);
    void writeToConsole(DVCOutputOption option);

private:
    Json::Value makeJsonBuffer(DVCOutputOption option);
    void doSortErrorInfos();

    std::vector<IDVCErrInfo*>    m_errorInfos;
    DVC_STRING                    m_outputFile;    
    Json::Value                    m_json;
};

