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
#include "DVCModule.h"

#include <iostream>
#include "../Util.h"

#include "../CommandParser.h"
#include "../Factory.h"

DVCModule::DVCModule(/*DVC_LPCSTR dvcFile, DVC_LPCSTR hwpxFile, bool printOutput*/)
{
    //m_dvcFile = dvcFile;
    //m_hwpxFile = hwpxFile;

    m_checker = NULL;

    //m_printOutput = printOutput;
    m_checkAll = true;
}

DVCModule::~DVCModule()
{
    if (m_checker)
        delete m_checker;
    m_checker = NULL;

    Factory::deleteObject(m_dvcOutput);
}

DVC_LPCSTR DVCModule::setCommand(int argc, DVC_CHAR* argv[])
{
    if (getCommandParser().commandParsing(argc, argv) == false)
        return getCommandParser().getMessage().c_str();
    return NULL;
}

void DVCModule::setOption(DVC::Options option, DVC_LPCSTR value)
{

}

void DVCModule::setJsonFilepath(DVC_LPCSTR filepath)
{

}

void DVCModule::setTargetFilepath(DVC_LPCSTR filepath)
{

}

bool DVCModule::doValidationCheck()
{
    bool bResult = false;    

    CommandParser comParser = getCommandParser();

    // is valid path
    if (comParser.getDVCFilepath().empty() == true || comParser.getTargetFilepath().empty() == true) {
        m_dvcOutput = Factory::getOutput(DVC::OutputType::JSON);
    }
    else {    // default
        m_dvcOutput = Factory::getOutput(DVC::OutputType::JSON);
    }

    DVC_STRING dvcFile = comParser.getDVCFilepath();

    bResult = Util::getAbsolutePath(dvcFile, dvcFile);
    if (!bResult)
    {
        std::cout << "Json 파일이 잘못되었습니다. 확인 부탁드립니다." << std::endl;
        return false;
    }

    DVC_STRING hwpxFile = comParser.getTargetFilepath();

    bResult = Util::getAbsolutePath(hwpxFile, hwpxFile);
    if (!bResult)
    {
        std::cout << "HWPX 파일이 잘못되었습니다. 확인 부탁드립니다." << std::endl;
        return false;
    }

    if (!m_checker)
        m_checker = new Checker(dvcFile, hwpxFile);

    if (!m_checker || !m_checker->Initialize())
    {
        std::cout << "초기화 진행에 오류가 있습니다. 입력된 파일의 오류를 확인하세요." << std::endl;
        return false;
    }

    m_checker->SetOutput(m_dvcOutput);

    m_checkAll = comParser.isEnableOption(DVC::Options::LevelAll);
        
    bResult = m_checker->CheckCharShape();

    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckCharShape. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckParaShape();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckParaShape. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bool bCheckDetail = false;
    if (comParser.GetOutputOption() == DVC::DVCOutputOption::TableDetail)
        bCheckDetail = true;

    bResult = m_checker->CheckTable(bCheckDetail);
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckTable. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckSpacialCharacter();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckSpacialCharacter. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckBullet();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckBullet. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckParaNumBullet();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckParaNumBullet. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckStyle();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckStyle. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    bResult = m_checker->CheckHyperlink();    
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckHyperlink. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }    

    bResult = m_checker->CheckMacro();
    if (!m_checkAll && !bResult)
    {
        std::cout << "Error CheckMacro. 오류 부분을 확인하세요." << std::endl;
        return bResult;
    }

    m_checker->PrintCheckedErrorList();    
    
    return true;
}

bool DVCModule::output()
{
    bool bRet = false;

    if (m_dvcOutput != NULL) {
        bool outputRun = false;

        // To File
        if (getCommandParser().isEnableOption(DVC::Options::OutputFile) == true) {
            const DVC_STRING* value = getCommandParser().getOptionValue(DVC::Options::OutputFile);
            if (value != NULL) {
                m_dvcOutput->writeToFile(*value, getCommandParser().GetOutputOption());
                bRet |= true;
                outputRun = true;
            }
        }

        // Default & To Console
        if (outputRun == false || getCommandParser().isEnableOption(DVC::Options::OutputConsole) == true) {
            bRet |= output(DVC::Options::OutputConsole);
        }
    }

    return bRet;
}

bool DVCModule::output(DVC::Options outputOption, DVC_LPCSTR filePath)
{    
    if (m_dvcOutput != NULL) {
        if (outputOption == DVC::Options::OutputConsole) {
            m_dvcOutput->writeToConsole(getCommandParser().GetOutputOption());
            return true;
        }
        else if (getCommandParser().isEnableOption(DVC::Options::OutputFile) == true) {
            m_dvcOutput->writeToFile(filePath, getCommandParser().GetOutputOption());
            return true;
        }
    }

    return false;
}

