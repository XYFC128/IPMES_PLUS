# for installing necessary packages 
#!pip install strenum
#!pip install pyvis

#!pip list
#!pip uninstall scipy -y
#!pip install scipy==1.8.0

#!pip uninstall networkx -y
#!pip install networkx==2.8.4

# basic
from abc import ABC, abstractmethod
import os,collections
import json,re
import numpy as np
from enum import Enum,auto
from strenum import StrEnum

# performance
from tqdm import tqdm

# graph
import networkx as nx
from pyvis.network import Network 

class NodeEdgeUtility:
    @staticmethod
    def getId(obj:dict):
        return obj['id']
    @staticmethod
    def getAllAttr(obj:dict):
        return obj['properties']
    @staticmethod
    def getAttr(obj:dict, key:str):
        return obj['properties'].get(key,"")
    @staticmethod
    def hasAttr(obj:dict, key:str):
        return (key in obj['properties'])
    @staticmethod
    def getEdgeFromNodeId(edge:dict):
        return edge['start']['id']
    @staticmethod
    def getEdgeToNodeId(edge:dict):
        return edge['end']['id']
    
    class Type(StrEnum):
        NODE_OTHER = auto()
        NODE_PROCESS = auto()
        NODE_CHAR = auto()
        NODE_FILE = auto()
        NODE_DIRECTORY = auto()
        NODE_SOCKET = auto()
        NODE_UNNAMED_PIPE = auto()
        NODE_SRCSINK = auto()
        NODE_REGISTRY = auto()
        EDGE_OTHER = auto()
    
    @staticmethod
    def transformPropertiesFieldValuesToString(obj:dict):
        for k,v in obj['properties'].items():
            if isinstance(v,int):
                obj['properties'][k] = str(v)
            elif isinstance(v,float):
                obj['properties'][k] = str(v)
            #if not(isinstance(v,str)):
            #    obj['properties'][k] = str(v)
        return obj
    
    __lineMaxChar = 50
    @staticmethod
    def conciseAttrToLines(conciseAttr:dict,withKey:bool=False,concatenateConciseAttr:dict={})->str:
        buffer = []
        for key,value in conciseAttr.items():
            if not(isinstance(value,str)):
                value = str(value)
            if key in concatenateConciseAttr:
                value = value + " || " + concatenateConciseAttr[key]
            remainChar = NodeEdgeUtility.__lineMaxChar
            if withKey:
                string = f"{key}: "
                buffer.append(string)
                remainChar -= len(string)
            remainValue = str(value)
            while True:
                if len(remainValue) > remainChar:
                    buffer.append(f'{remainValue[:(remainChar)]}\n')
                    remainValue = remainValue[(remainChar):]
                    remainChar = NodeEdgeUtility.__lineMaxChar
                else:
                    buffer.append(f'{remainValue}\n')
                    break
        return ''.join(buffer)
     
class Unitwalk(ABC): 
    class Part(Enum):
        NODE_FROM = 0
        NODE_TO = 1
        EDGE = 2
    
    class Info(Enum):
        ID = 0
        PID = 11
        PPID = 12
        PATH = 13
        ADDR = 14
        PROCNAME = 15
        FILENAME = 16

    def __init__(self, nodeFrom:dict, nodeTo:dict, edge:dict):
        self.__nodeFrom = NodeEdgeUtility.transformPropertiesFieldValuesToString(nodeFrom)
        self.__nodeTo = NodeEdgeUtility.transformPropertiesFieldValuesToString(nodeTo)
        self.__edge = NodeEdgeUtility.transformPropertiesFieldValuesToString(edge)
        return
    
    def __str__(self):
        return f'{self.__nodeFrom}\n{self.__edge}\n{self.__nodeTo}\n'
    
    def getAttr(self, part:Part, key:str):
        if part == Unitwalk.Part.NODE_FROM:
            return self.__nodeFrom['properties'].get(key,"")
        elif part == Unitwalk.Part.NODE_TO:
            return self.__nodeTo['properties'].get(key,"")
        elif part == Unitwalk.Part.EDGE:
            return self.__edge['properties'].get(key,"")
        else:
            raise Exception(f'the part argument {part} is out of range')
    
    def hasAttr(self, part:Part, key:str)->bool:
        if part == Unitwalk.Part.NODE_FROM:
            return (key in self.__nodeFrom['properties'])
        elif part == Unitwalk.Part.NODE_TO:
            return (key in self.__nodeTo['properties'])
        elif part == Unitwalk.Part.EDGE:
            return (key in self.__edge['properties'])
        else:
            raise Exception(f'the part argument {part} is out of range')
    
    def getId(self, part:Part):
        if part == Unitwalk.Part.NODE_FROM:
            return self.__nodeFrom['id']
        elif part == Unitwalk.Part.NODE_TO:
            return self.__nodeTo['id']
        elif part == Unitwalk.Part.EDGE:
            return self.__edge['id']
        else:
            raise Exception(f'the part argument {part} is out of range')
        
    def getInfo(self, part:Part, info:Info):
        raise Exception(f'The default getInfo function is not implemented, please overwrite a new function')
    
    def toLogFormat(self)->dict:
        log = {}
        for (partKey,value) in zip(['m','r','n'],[self.__nodeFrom, self.__edge, self.__nodeTo]):
            log[partKey] = value
        return log
    
    @staticmethod
    def getNodeSignature(node:dict)->tuple:
        raise Exception(f'The default getNodeSignature function is not implemented, please overwrite a new function')
    
    @staticmethod
    def getEdgeSignature(edge:dict)->tuple:
        raise Exception(f'The default getNodeSignature function is not implemented, please overwrite a new function')
    
    def getType(self, part)->tuple:
        if part == Unitwalk.Part.NODE_FROM:
            return self.getNodeSignature(self.__nodeFrom)
        elif part == Unitwalk.Part.NODE_TO:
            return self.getNodeSignature(self.__nodeTo)
        elif part == Unitwalk.Part.EDGE:
            return self.getEdgeSignature(self.__edge)
        else:
            raise Exception(f'the part argument {part} is out of range')
    
    # in seconds
    def getEdgeTime(self)->tuple[float,float]:
        raise Exception(f'The default getEdgeTime function is not implemented, please overwrite a new function')

    def getConciseAttr(self, part:Part, detail:bool=False)->dict:
        if part == Unitwalk.Part.NODE_FROM:
            return self.getNodeConciseAttr(self.__nodeFrom, detail)
        elif part == Unitwalk.Part.NODE_TO:
            return self.getNodeConciseAttr(self.__nodeTo, detail)
        elif part == Unitwalk.Part.EDGE:
            return self.getEdgeConciseAttr(self.__edge, detail)
        else:
            raise Exception(f'the part argument {part} is out of range')
    
    def getRawdataInCopy(self, part:Part)->dict:
        if part == Unitwalk.Part.NODE_FROM:
            return dict(self.__nodeFrom)
        elif part == Unitwalk.Part.NODE_TO:
            return dict(self.__nodeTo)
        elif part == Unitwalk.Part.EDGE:
            return dict(self.__edge)
        else:
            raise Exception(f'the part argument {part} is out of range')
        
    @staticmethod
    def getNodeConciseAttr(node:dict, detail:bool)->dict:
        raise Exception(f'The default getNodeConciseAttr function is not implemented, please overwrite a new function')
    
    @staticmethod
    def getEdgeConciseAttr(edge:dict, detail:bool)->dict:
        raise Exception(f'The default getEdgeConciseAttr function is not implemented, please overwrite a new function')
        
class SPADE_Unitwalk(Unitwalk):
    
    def __init__(self, nodeFrom:dict, nodeTo:dict, edge:dict):
        super().__init__(nodeFrom, nodeTo, edge)
        return
    
    def getInfo(self, part:Unitwalk.Part, info:Unitwalk.Info):
        match info:
            case Unitwalk.Info.PID:
                return self.getAttr(part, "pid")
            case Unitwalk.Info.PPID:
                return self.getAttr(part, "ppid")
            case Unitwalk.Info.PATH:
                return self.getAttr(part, "path")
            case Unitwalk.Info.ADDR:
                return f'{self.getAttr(part, "remote address")}:{self.getAttr(part, "remote port")}'
            case Unitwalk.Info.PROCNAME:
                return self.getAttr(part, "name")
            case Unitwalk.Info.FILENAME:
                # not be implemented
                return ""
            case Unitwalk.Info._:
                raise Exception(f'the info argument {info} is not implemented')
            
    @staticmethod
    def getNodeSignature(node:dict)->tuple:
        abstractType = None
        valueList = []
        value = NodeEdgeUtility.getAttr(node,'type')
        if value == 'Process':
            abstractType = NodeEdgeUtility.Type.NODE_PROCESS
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'name'))
        elif value == 'Artifact':
            value = NodeEdgeUtility.getAttr(node,'subtype')
            if value == 'file':
                abstractType = NodeEdgeUtility.Type.NODE_FILE
                valueList.append(value)
                valueList.append(NodeEdgeUtility.getAttr(node,'path'))
            elif value == 'directory':
                abstractType = NodeEdgeUtility.Type.NODE_DIRECTORY
                valueList.append(value)
                valueList.append(NodeEdgeUtility.getAttr(node,'path'))
            elif value == 'network socket':
                abstractType = NodeEdgeUtility.Type.NODE_SOCKET
                valueList.append(value)
                valueList.append(NodeEdgeUtility.getAttr(node,'remote address'))
                valueList.append(NodeEdgeUtility.getAttr(node,'remote port'))
            elif value == 'unnamed pipe':
                abstractType = NodeEdgeUtility.Type.NODE_UNNAMED_PIPE
                valueList.append(value)
            else:
                abstractType = NodeEdgeUtility.Type.NODE_OTHER
                valueList.append(value)
        else:
            abstractType = NodeEdgeUtility.Type.NODE_OTHER
            valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    @staticmethod
    def getEdgeSignature(edge:dict)->tuple:
        abstractType = None
        valueList = []
        abstractType = NodeEdgeUtility.Type.EDGE_OTHER
        value = NodeEdgeUtility.getAttr(edge,'operation')
        valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    def getEdgeTime(self)->tuple[float,float]:
        if self.hasAttr(Unitwalk.Part.EDGE,'time'):
            edgeTime = float(self.getAttr(Unitwalk.Part.EDGE,'time'))
            return (edgeTime,edgeTime)
        else:
            edgeEarliestTime = float(self.getAttr(Unitwalk.Part.EDGE,'earliest'))
            edgeLastestTime = float(self.getAttr(Unitwalk.Part.EDGE,'lastest'))
            return (edgeEarliestTime,edgeLastestTime)
    
    @staticmethod
    def getNodeConciseAttr(node:dict, detail:bool)->dict:
        conciseAttrDict = {}
        nodeType = NodeEdgeUtility.getAttr(node,'type')
        if nodeType == 'Process':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['pid'] = NodeEdgeUtility.getAttr(node,'pid')
                conciseAttrDict['ppid'] = NodeEdgeUtility.getAttr(node,'ppid')
                conciseAttrDict['start time'] = NodeEdgeUtility.getAttr(node,'start time')
                conciseAttrDict['cmd'] = NodeEdgeUtility.getAttr(node,'command line')
            else:
                conciseAttrDict['type'] = 'Process'
                conciseAttrDict['uid'] = 'user ' + NodeEdgeUtility.getAttr(node,'uid')
                conciseAttrDict['name'] = NodeEdgeUtility.getAttr(node,'name')

        elif nodeType == 'Artifact':
            nodeSubType = NodeEdgeUtility.getAttr(node,'subtype')
            if nodeSubType == 'file':
                if detail:
                    conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                    conciseAttrDict['inode'] = NodeEdgeUtility.getAttr(node,'inode')
                    conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'version')
                else:
                    conciseAttrDict['type'] = 'File'
                    conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'path')
            elif nodeSubType == 'directory':
                if detail:
                    conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                    conciseAttrDict['inode'] = NodeEdgeUtility.getAttr(node,'inode')
                    conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'version')
                else:
                    conciseAttrDict['type'] = 'Directory'
                    conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'path')
            elif nodeSubType == 'network socket':
                if detail:
                    conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                    conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'version')
                    conciseAttrDict['protocol'] = NodeEdgeUtility.getAttr(node,'protocol')
                else:
                    conciseAttrDict['type'] = 'Socket'
                    conciseAttrDict['addr'] = f"{NodeEdgeUtility.getAttr(node,'remote address')}:{NodeEdgeUtility.getAttr(node,'remote port')}"
            elif nodeSubType == 'unnamed pipe':
                if detail:
                    conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                    conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'version')
                else:
                    conciseAttrDict['type'] = 'UnnamedPipe'

        return conciseAttrDict
        
    @staticmethod
    def getEdgeConciseAttr(edge:dict, detail:bool)->dict:
        conciseAttrDict = {}
        if detail:
            conciseAttrDict['id'] = NodeEdgeUtility.getId(edge)
            if NodeEdgeUtility.hasAttr(edge,'time'):
                edgeTime = float(NodeEdgeUtility.getAttr(edge,'time'))
                conciseAttrDict['time'] = f'({edgeTime},{edgeTime})'
            else:
                edgeEarliestTime = float(NodeEdgeUtility.getAttr(edge,'earliest'))
                edgeLastestTime = float(NodeEdgeUtility.getAttr(edge,'lastest'))
                conciseAttrDict['time'] = f'({edgeEarliestTime},{edgeLastestTime})'
        else:
            conciseAttrDict['type'] = NodeEdgeUtility.getAttr(edge,'operation')
        return conciseAttrDict
    
    
class DARPA_Unitwalk(Unitwalk):
    
    def __init__(self, nodeFrom:dict, nodeTo:dict, edge:dict):
        super().__init__(nodeFrom, nodeTo, edge)
        return
    
    def getInfo(self, part:Unitwalk.Part, info:Unitwalk.Info):
        match info:
            case Unitwalk.Info.PID:
                return self.getAttr(part, "Subject_cid")
            case Unitwalk.Info.PPID:
                return self.getAttr(part, "Subject_properties_map_ppid")
            case Unitwalk.Info.PATH:
                return self.getAttr(part, "path")
            case Unitwalk.Info.ADDR:
                return f'{self.getAttr(part, "NetFlowObject_remoteAddress")}:{self.getAttr(part, "NetFlowObject_remotePort")}'
            case Unitwalk.Info.PROCNAME:
                return self.getAttr(part, "Subject_properties_map_name")
            case Unitwalk.Info.FILENAME:
                fileName = self.getAttr(part, "path")
                mathces = re.findall("(?<=\\/)[^\\/.]+(?:(?=$)|(?=\\.[^\\/]*$))",fileName)
                fileName = mathces[0] if len(mathces)>0 else ""
                return fileName
            case Unitwalk.Info._:
                raise Exception(f'the info argument {info} is not implemented')
    
    @staticmethod
    def getNodeSignature(node:dict)->tuple:
        abstractType = None
        valueList = []
        value = None
        if NodeEdgeUtility.hasAttr(node,'NetFlowObject_baseObject_epoch'):
            value = 'OBJECT_SOCKET'
        elif NodeEdgeUtility.hasAttr(node,'UnnamedPipeObject_baseObject_epoch'):
            value = 'OBJECT_UNNAMEPIPE'
        else:
            value = NodeEdgeUtility.getAttr(node,'type')
        if value == 'SUBJECT_PROCESS' or value == 'SUBJECT_UNIT':
            abstractType = NodeEdgeUtility.Type.NODE_PROCESS
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'Subject_properties_map_name'))
        elif value == 'FILE_OBJECT_CHAR':
            abstractType = NodeEdgeUtility.Type.NODE_CHAR
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'path'))
        elif value == 'FILE_OBJECT_FILE':
            abstractType = NodeEdgeUtility.Type.NODE_FILE
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'path'))
        elif value == 'FILE_OBJECT_DIR':
            abstractType = NodeEdgeUtility.Type.NODE_DIRECTORY
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'path'))
        elif value == 'OBJECT_SOCKET':
            abstractType = NodeEdgeUtility.Type.NODE_SOCKET
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'NetFlowObject_remoteAddress'))
            valueList.append(NodeEdgeUtility.getAttr(node,'NetFlowObject_remotePort'))
        elif value == 'OBJECT_UNNAMEPIPE':
            abstractType = NodeEdgeUtility.Type.NODE_UNNAMED_PIPE
            valueList.append(value)
        elif value == 'SRCSINK_UNKNOWN':
            abstractType = NodeEdgeUtility.Type.NODE_SRCSINK
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'SrcSinkObject_pid'))
        else:
            abstractType = NodeEdgeUtility.Type.NODE_OTHER
            valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    @staticmethod
    def getEdgeSignature(edge:dict)->tuple:
        abstractType = None
        valueList = []
        abstractType = NodeEdgeUtility.Type.EDGE_OTHER
        value = NodeEdgeUtility.getAttr(edge,'type')
        valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    def getEdgeTime(self)->tuple[float,float]:
        edgeEarliestTime = float(self.getAttr(Unitwalk.Part.EDGE,'earliest')) / 1000000000.0
        edgeLastestTime = float(self.getAttr(Unitwalk.Part.EDGE,'lastest')) / 1000000000.0
        return (edgeEarliestTime,edgeLastestTime)
    
    @staticmethod
    def getNodeConciseAttr(node:dict, detail:bool)->dict:
        conciseAttrDict = {}
        if NodeEdgeUtility.hasAttr(node,'NetFlowObject_baseObject_epoch'):
            nodeType = 'OBJECT_SOCKET'
        elif NodeEdgeUtility.hasAttr(node,'UnnamedPipeObject_baseObject_epoch'):
            nodeType = 'OBJECT_UNNAMEPIPE'
        else:
            nodeType = NodeEdgeUtility.getAttr(node,'type')
        if nodeType == 'SUBJECT_PROCESS' or nodeType == 'SUBJECT_UNIT':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['pid'] = NodeEdgeUtility.getAttr(node,'Subject_cid')
                conciseAttrDict['ppid'] = NodeEdgeUtility.getAttr(node,'Subject_properties_map_ppid')
                conciseAttrDict['start time'] = float(NodeEdgeUtility.getAttr(node,'Subject_startTimestampNanos')) / 1000000000.0
                conciseAttrDict['cmd'] = NodeEdgeUtility.getAttr(node,'Subject_cmdLine')
            else:
                conciseAttrDict['type'] = 'Process'
                conciseAttrDict['uid'] = 'user X'
                conciseAttrDict['name'] = NodeEdgeUtility.getAttr(node,'Subject_properties_map_name')
        
        elif nodeType == 'FILE_OBJECT_CHAR':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'FileObject_baseObject_epoch')
                conciseAttrDict['UUID'] = NodeEdgeUtility.getAttr(node,'UUID')
            else:
                conciseAttrDict['type'] = 'CHAR'
                conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'path')
        
        elif nodeType == 'FILE_OBJECT_FILE':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'FileObject_baseObject_epoch')
                conciseAttrDict['UUID'] = NodeEdgeUtility.getAttr(node,'UUID')
            else:
                conciseAttrDict['type'] = 'FILE'
                conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'path')

        elif nodeType == 'FILE_OBJECT_DIR':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'FileObject_baseObject_epoch')
                conciseAttrDict['UUID'] = NodeEdgeUtility.getAttr(node,'UUID')
            else:
                conciseAttrDict['type'] = 'Directory'
                conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'path')

        elif nodeType == 'OBJECT_SOCKET':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'NetFlowObject_baseObject_epoch')
                conciseAttrDict['protocol'] = NodeEdgeUtility.getAttr(node,'NetFlowObject_ipProtocol')
            else:
                conciseAttrDict['type'] = 'Socket'
                conciseAttrDict['addr'] = f"{NodeEdgeUtility.getAttr(node,'NetFlowObject_remoteAddress')}:{NodeEdgeUtility.getAttr(node,'NetFlowObject_remotePort')}"
        elif nodeType == 'OBJECT_UNNAMEPIPE':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'UnnamedPipeObject_baseObject_epoch')
            else:
                conciseAttrDict['type'] = 'UnnamedPipe'
        elif nodeType == 'SRCSINK_UNKNOWN':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['version'] = NodeEdgeUtility.getAttr(node,'SrcSinkObject_epoch')
                conciseAttrDict['pid'] = NodeEdgeUtility.getAttr(node,'SrcSinkObject_pid')
            else:
                conciseAttrDict['type'] = 'SrcSink'
        return conciseAttrDict
    @staticmethod
    def getEdgeConciseAttr(edge:dict, detail:bool)->dict:
        conciseAttrDict = {}
        if detail:
            conciseAttrDict['id'] = NodeEdgeUtility.getId(edge)
            edgeEarliestTime = float(NodeEdgeUtility.getAttr(edge,'earliest')) / 1000000000.0
            edgeLastestTime = float(NodeEdgeUtility.getAttr(edge,'lastest')) / 1000000000.0
            conciseAttrDict['time'] = f'({edgeEarliestTime},{edgeLastestTime})'
        else:
            conciseAttrDict['type'] = NodeEdgeUtility.getAttr(edge,'type')[6:]
        return conciseAttrDict

class HighLevel_Unitwalk(Unitwalk):
    
    def __init__(self, nodeFrom:dict, nodeTo:dict, edge:dict):
        super().__init__(nodeFrom, nodeTo, edge)
        return
    
    def getInfo(self, part:Unitwalk.Part, info:Unitwalk.Info):
        raise Exception(f'The default getInfo function is not implemented, please overwrite a new function')

    @staticmethod
    def getNodeSignature(node:dict)->tuple:
        abstractType = None
        valueList = []
        value = NodeEdgeUtility.getAttr(node,'level')
        if value == "L1":
            abstractType = NodeEdgeUtility.Type.NODE_OTHER
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'name'))
        return (abstractType,) + tuple(valueList)
    
    @staticmethod
    def getEdgeSignature(edge:dict)->tuple:
        abstractType = None
        valueList = []
        abstractType = NodeEdgeUtility.Type.EDGE_OTHER
        value = NodeEdgeUtility.getAttr(edge,'correlation')
        valueList.append(value)
        # other info
        return (abstractType,) + tuple(valueList)
    
    def getEdgeTime(self)->tuple[float,float]:
        if self.hasAttr(Unitwalk.Part.EDGE,'time'):
            edgeTime = float(self.getAttr(Unitwalk.Part.EDGE,'time'))
            return (edgeTime,edgeTime)
        else:
            edgeEarliestTime = float(self.getAttr(Unitwalk.Part.EDGE,'earliest'))
            edgeLastestTime = float(self.getAttr(Unitwalk.Part.EDGE,'lastest'))
            return (edgeEarliestTime,edgeLastestTime)
    
    @staticmethod
    def getNodeConciseAttr(node:dict, detail:bool)->dict:
        conciseAttrDict = {}
        nodeLevel = NodeEdgeUtility.getAttr(node,'level')
        nodeName = NodeEdgeUtility.getAttr(node,'name')
        nodeNodeIDSet = NodeEdgeUtility.getAttr(node,'nodeIDSet')
        nodeEdgeIDSet = NodeEdgeUtility.getAttr(node,'edgeIDSet')
        if nodeLevel == 'L1':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                nodeEarliestTime = NodeEdgeUtility.getAttr(node,'earliest')
                nodeLastestTime = NodeEdgeUtility.getAttr(node,'lastest')
                conciseAttrDict['time'] = f'({nodeEarliestTime},{nodeLastestTime})'
                # edgeID/nodeID set
                conciseAttrDict['nodeIDSet'] = nodeNodeIDSet
                conciseAttrDict['edgeIDSet'] = nodeEdgeIDSet
                # pid/ppid/path/addr set
                conciseAttrDict['pidTable'] = NodeEdgeUtility.getAttr(node,'pidTable')
                conciseAttrDict['ppidTable'] = NodeEdgeUtility.getAttr(node,'ppidTable')
                conciseAttrDict['pathTable'] = NodeEdgeUtility.getAttr(node,'pathTable')
                conciseAttrDict['addrTable'] = NodeEdgeUtility.getAttr(node,'addrTable')
            else:
                conciseAttrDict['level'] = nodeLevel
                conciseAttrDict['name'] = nodeName

        return conciseAttrDict
        
    @staticmethod
    def getEdgeConciseAttr(edge:dict, detail:bool)->dict:
        conciseAttrDict = {}
        if detail:
            conciseAttrDict['id'] = NodeEdgeUtility.getId(edge)
            conciseAttrDict['nodeIDTupleList'] = NodeEdgeUtility.getAttr(edge,'nodeIDTupleList')
            if NodeEdgeUtility.hasAttr(edge,'time'):
                edgeTime = float(NodeEdgeUtility.getAttr(edge,'time'))
                conciseAttrDict['time'] = f'({edgeTime},{edgeTime})'
            else:
                edgeEarliestTime = float(NodeEdgeUtility.getAttr(edge,'earliest'))
                edgeLastestTime = float(NodeEdgeUtility.getAttr(edge,'lastest'))
                conciseAttrDict['time'] = f'({edgeEarliestTime},{edgeLastestTime})'
        else:
            conciseAttrDict['correlation'] = NodeEdgeUtility.getAttr(edge,'correlation')
        return conciseAttrDict
    
    
class PROCMON_Unitwalk(Unitwalk):
    
    def __init__(self, nodeFrom:dict, nodeTo:dict, edge:dict):
        super().__init__(nodeFrom, nodeTo, edge)
        return
    
    @staticmethod
    def getNodeSignature(node:dict)->tuple:
        abstractType = None
        valueList = []
        value = NodeEdgeUtility.getAttr(node,'Type')
        if value == 'Process':
            abstractType = NodeEdgeUtility.Type.NODE_PROCESS
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'Name'))
        elif value == 'File':
            abstractType = NodeEdgeUtility.Type.NODE_FILE
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'Name'))
        elif value == 'Network':
            abstractType = NodeEdgeUtility.Type.NODE_SOCKET
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'Dstaddress'))
        elif value == 'Registry':
            abstractType = NodeEdgeUtility.Type.NODE_REGISTRY
            valueList.append(value)
            valueList.append(NodeEdgeUtility.getAttr(node,'Key'))
        else:
            abstractType = NodeEdgeUtility.Type.NODE_OTHER
            valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    @staticmethod
    def getEdgeSignature(edge:dict)->tuple:
        abstractType = None
        valueList = []
        abstractType = NodeEdgeUtility.Type.EDGE_OTHER
        value = NodeEdgeUtility.getAttr(edge,'operation')
        valueList.append(value)
        return (abstractType,) + tuple(valueList)
    
    def getEdgeTime(self)->tuple[float,float]:
        if self.hasAttr(Unitwalk.Part.EDGE,'time'):
            edgeTime = float(self.getAttr(Unitwalk.Part.EDGE,'time'))
            return (edgeTime,edgeTime)
        else:
            edgeEarliestTime = float(self.getAttr(Unitwalk.Part.EDGE,'earliest'))
            edgeLastestTime = float(self.getAttr(Unitwalk.Part.EDGE,'lastest'))
            return (edgeEarliestTime,edgeLastestTime)
    
    @staticmethod
    def getNodeConciseAttr(node:dict, detail:bool)->dict:
        conciseAttrDict = {}
        nodeType = NodeEdgeUtility.getAttr(node,'Type')
        if nodeType == 'Process':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                conciseAttrDict['pid'] = NodeEdgeUtility.getAttr(node,'Pid')
                #conciseAttrDict['ppid'] = NodeEdgeUtility.getAttr(node,'')
                conciseAttrDict['cmd'] = NodeEdgeUtility.getAttr(node,'Cmdline')
            else:
                conciseAttrDict['type'] = 'Process'
                conciseAttrDict['name'] = NodeEdgeUtility.getAttr(node,'Name')
        
        elif nodeType == 'File':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
            else:
                conciseAttrDict['type'] = 'File'
                conciseAttrDict['path'] = NodeEdgeUtility.getAttr(node,'Name')
        
        elif nodeType == 'Network':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
            else:
                conciseAttrDict['type'] = 'Socket'
                conciseAttrDict['addr'] = NodeEdgeUtility.getAttr(node,'Dstaddress')
        elif nodeType == 'Registry':
            if detail:
                conciseAttrDict['id'] = NodeEdgeUtility.getId(node)
                if NodeEdgeUtility.hasAttr(node,'Value'):
                    conciseAttrDict['value'] = NodeEdgeUtility.getAttr(node,'value')
            else:
                conciseAttrDict['type'] = 'Registry'
                conciseAttrDict['key'] = NodeEdgeUtility.getAttr(node,'Key')
        return conciseAttrDict
    @staticmethod
    def getEdgeConciseAttr(edge:dict, detail:bool)->dict:
        conciseAttrDict = {}
        if detail:
            conciseAttrDict['id'] = NodeEdgeUtility.getId(edge)
            edgeTime = NodeEdgeUtility.getAttr(edge,'time')
            conciseAttrDict['time'] = f'({edgeTime},{edgeTime})'
        else:
            conciseAttrDict['type'] = NodeEdgeUtility.getAttr(edge,'operation')
        return conciseAttrDict    

class TransformUtility:
    """
    returen a generator of all unitwalks
    """
    @staticmethod
    def createReadUnitwalksGenerator(unitWalkClass=Unitwalk, logFilePath:str=None, logWalks:list=None,
                                     endWithNone:bool=False, withTqdm:bool=False
                                    ):
        if logWalks is None:
            if logFilePath is None:
                raise Exception("logFilePath is None")
            if withTqdm:
                def readUnitwalksGenerator():
                    logFile = None
                    fileSize = os.stat(logFilePath).st_size # in Byte
                    pbar = tqdm(total=fileSize)
                    try:
                        logFile = None
                        logFile = open(logFilePath,mode='rt',encoding='utf-8')
                        for line in logFile:
                            logWalk = None
                            pbar.update(len(line))
                            if not(line.startswith('#')):
                                log = json.loads(line)  # dict
                                fromNode = log['m']
                                edge = log['r']
                                toNode = log['n']
                                logWalk = unitWalkClass(fromNode, toNode, edge)
                                yield logWalk
                    finally:
                        if logFile is not None:
                            logFile.close()
                        if pbar is not None:
                            pbar.close()
            else:
                def readUnitwalksGenerator():
                    logFile = None
                    try:
                        logFile = None
                        logFile = open(logFilePath,mode='rt',encoding='utf-8')
                        for line in logFile:
                            logWalk = None
                            if not(line.startswith('#')):
                                log = json.loads(line)  # dict
                                fromNode = log['m']
                                edge = log['r']
                                toNode = log['n']
                                logWalk = unitWalkClass(fromNode, toNode, edge)
                                yield logWalk
                    finally:
                        if logFile is not None:
                            logFile.close()
        else:
            if withTqdm:
                def readUnitwalksGenerator():
                    for logWalk in tqdm(logWalks):
                        yield logWalk
            else:
                def readUnitwalksGenerator():
                    for logWalk in logWalks:
                        yield logWalk
        return readUnitwalksGenerator()
    
    @staticmethod
    def transformTraceToNetworkxGraph(readUnitwalksGenerator)->nx.MultiDiGraph:
        graph = nx.MultiDiGraph()
        for unitwalk in readUnitwalksGenerator:
            fromNode = unitwalk.getRawdataInCopy(Unitwalk.Part.NODE_FROM)
            edge = unitwalk.getRawdataInCopy(Unitwalk.Part.EDGE)
            toNode = unitwalk.getRawdataInCopy(Unitwalk.Part.NODE_TO)
            
            #NodeEdgeUtility.getAllAttr(edge)['id'] = NodeEdgeUtility.getId(edge)
            fromNodeId = NodeEdgeUtility.getId(fromNode)
            toNodeId = NodeEdgeUtility.getId(toNode)
            if fromNodeId not in graph:
                graph.add_nodes_from([(fromNodeId,fromNode)])
            if toNodeId not in graph:
                graph.add_nodes_from([(toNodeId,toNode)])
            graph.add_edges_from([(fromNodeId,toNodeId,edge)])
        #print(list(graph.nodes(data=True))[0])
        return graph
    
    @staticmethod
    def transformNetworkxGraphForNeuroMatch(graph:nx.MultiDiGraph, toUndirected=True,
                                            nodeAttrsTransformer=(lambda nodeAttrs: nodeAttrs), edgeAttrsTransformer=(lambda edgeAttrsList: {"attrs":edgeAttrsList})):
        nodeTable = dict(graph.nodes(data=True))
        newGraph = None
        if toUndirected:
            newGraph = nx.Graph()
            for nodeId,nodeData in nodeTable.items():
                if nodeId not in newGraph:
                    newGraph.add_nodes_from([(nodeId,nodeAttrsTransformer(nodeData))])
                for adjNodeId,edgesView in graph.adj[nodeId].items():
                    if adjNodeId not in newGraph:
                        newGraph.add_nodes_from([(adjNodeId,nodeAttrsTransformer(nodeTable[adjNodeId]))])
                    if (nodeId,adjNodeId) not in newGraph.edges:
                        # integrate edges in double way
                        edgeAttrsList = list(edgesView.values())
                        if ((adjNodeId,nodeId) in graph.edges) and (nodeId != adjNodeId):
                            edgeAttrsList = edgeAttrsList + list(graph[adjNodeId][nodeId].values())
                        newGraph.add_edges_from([(nodeId,adjNodeId,edgeAttrsTransformer(edgeAttrsList))])
        else:
            newGraph = nx.DiGraph()
            for nodeId,nodeData in nodeTable.items():
                if nodeId not in newGraph:
                    newGraph.add_nodes_from([(nodeId,nodeAttrsTransformer(nodeData))])
                for adjNodeId,edgesView in graph.adj[nodeId].items():
                    #print(f'{type(adjNodeId)} {adjNodeId} {type(nodeTable[adjNodeId])} {nodeTable[adjNodeId]}')
                    #print(f'{type(list(edgesView.values()))} {list(edgesView.values())}')
                    if adjNodeId not in newGraph:
                        newGraph.add_nodes_from([(adjNodeId,nodeAttrsTransformer(nodeTable[adjNodeId]))])
                    if (nodeId,adjNodeId) not in newGraph.edges:
                        # integrate edges in single way
                        newEdgeAttrs = edgeAttrsTransformer(list(edgesView.values()))
                        newGraph.add_edges_from([(nodeId,adjNodeId,newEdgeAttrs)])
        return newGraph
    
    @staticmethod
    def transformTraceToPyvisNetwork(readUnitwalksGenerator)->Network:
        network = Network(height='100%',width='60%',directed=True,notebook=False,layout=True)
        for unitwalk in readUnitwalksGenerator:
            fromNodeId = unitwalk.getId(Unitwalk.Part.NODE_FROM)
            toNodeId = unitwalk.getId(Unitwalk.Part.NODE_TO)
            if fromNodeId not in network.get_nodes():
                network.add_node(fromNodeId , \
                                 title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_FROM,detail=True),withKey=True), \
                                 label = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_FROM,detail=False),withKey=False), \
                                 shape = 'box', \
                                 physics = True \
                                )
            if toNodeId not in network.get_nodes():
                network.add_node(toNodeId , \
                                 title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_TO,detail=True),withKey=True), \
                                 label = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_TO,detail=False),withKey=False), \
                                 shape = 'box', \
                                 physics = True \
                                )
            network.add_edge(fromNodeId, \
                             toNodeId, \
                             title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.EDGE,detail=True),withKey=True), \
                             label = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.EDGE,detail=False),withKey=False), \
                             physics = True \
                            )
        return network

class OrderReltionUtility:
    @staticmethod
    def createORelsFromJSON(oRelsFilePath:str, defaultBrotherAgeDiffLimit=float('inf'))->dict:
        oRels = None
        oRelsFile = None
        try:
            oRelsFile = open(oRelsFilePath,mode='rt',encoding='utf-8')
            line = oRelsFile.readline()
            oRels = json.loads(line)
        finally:
            if oRelsFile is not None:
                oRelsFile.close()
        
        newORels = {}
        # string key to integer
        # parents and children to list to set
        # brotherAgeDiffLimit -1 => float('inf')
        for (edgeId, oRel) in oRels.items():
            newORel = {}
            for (key, value) in oRel.items():
                if key == 'parents':
                    newORel[key] = tuple(value)
                elif key == 'children':
                    newORel[key] = tuple(value)
                elif key == 'brotherAgeDiffLimit':
                    if value < 0:
                        value = float('inf')
                    newORel[key] = value
                else:
                    newORel[key] = value
            if 'brotherAgeDiffLimit' not in newORel:
                newORel['brotherAgeDiffLimit'] = defaultBrotherAgeDiffLimit
            
            if edgeId == 'root':
                newORels[edgeId] = newORel
            else:
                newORels[int(edgeId)] = newORel
        
        # check validation
        visitedEdgeIdSet = set()
        def checkValidation(edgeId)->bool:
            nonlocal newORels
            nonlocal visitedEdgeIdSet
            if edgeId in visitedEdgeIdSet:
                return True
            visitedEdgeIdSet.add(edgeId)
            oRel = newORels[edgeId]
            for child in oRel['children']:
                if edgeId not in newORels[child]['parents']:
                    raise Exception(f'Error in edge {child} without parent edge {edgeId}')
                checkValidation(child)
            return True
        checkValidation('root')
        for edgeId in newORels.keys():
            if edgeId not in visitedEdgeIdSet:
                raise Exception(f'Error in edge {edgeId} without being visited')
                
        return newORels
    
    @staticmethod
    # for presentation
    def transformToPyvisNetwork(oRels:dict)->Network:
        network = Network(height='100%',width='60%',directed=True,notebook=False,layout=True)
        
        networkNodeIdSet = set()
        for edgeId,oRel in oRels.items():
            if edgeId not in networkNodeIdSet:
                network.add_node(edgeId , \
                                 title = str(edgeId), \
                                 label = str(edgeId), \
                                 shape = 'box', \
                                 physics = True \
                                )
                networkNodeIdSet.add(edgeId)
            for parent in oRel['parents']:
                if parent not in networkNodeIdSet:
                    network.add_node(parent , \
                                     title = str(parent), \
                                     label = str(parent), \
                                     shape = 'box', \
                                     physics = True \
                                    )
                    networkNodeIdSet.add(parent)
                network.add_edge(edgeId, \
                                 parent, \
                                 color = 'red', \
                                 physics = True \
                                )
            for child in oRel['children']:
                if child not in networkNodeIdSet:
                    network.add_node(child , \
                                     title = str(child), \
                                     label = str(child), \
                                     shape = 'box', \
                                     physics = True \
                                    )
                    networkNodeIdSet.add(child)
                network.add_edge(edgeId, \
                                 child, \
                                 color = 'blue', \
                                 physics = True \
                                )
         
        return network

class Pattern:
    
    class MatchUtility:
        
        @staticmethod
        def equalSigMatch(patternSig:tuple, targetSig:tuple)->bool:
            return patternSig == targetSig
            
        @staticmethod
        def regExSigMatch(patternSig:tuple, targetSig:tuple)->bool:
            if patternSig[0] != targetSig[0]:
                return False
            for patternStr,targetStr in zip(patternSig[1:], targetSig[1:]):
                if re.match('^' + patternStr + '$', targetStr) is None:
                    return False
            return True
        
        @staticmethod
        # default match, always return false
        def defaultMatch(patternWalk:Unitwalk, logWalk:Unitwalk)->bool:
            return False
        
        # ad-hoc matching by comparing node type, file path, process name ...
        @staticmethod
        def adHocMatch(patternWalk:Unitwalk, logWalk:Unitwalk)->bool:
            for part in [Unitwalk.Part.EDGE,Unitwalk.Part.NODE_FROM,Unitwalk.Part.NODE_TO]:
                if not(Pattern.MatchUtility.equalSigMatch(patternWalk.getType(part), logWalk.getType(part))):
                    return False
            return True
        
        # ad-hoc regular expression matching by comparing node type, file path, process name ... 
        @staticmethod
        def adHocRegExMatch(patternWalk:Unitwalk, logWalk:Unitwalk)->bool:
            for part in [Unitwalk.Part.EDGE,Unitwalk.Part.NODE_FROM,Unitwalk.Part.NODE_TO]:
                if not(Pattern.MatchUtility.regExSigMatch(patternWalk.getType(part), logWalk.getType(part))):
                    return False
            return True
        
        @staticmethod
        # for networkx graph matching, the matching is based on their sigature
        def nodeMatchHelper(unitWalkClass:type[Unitwalk], nodeSigMatch:type[lambda:None])->type[lambda:None]:
            getNodeSignature = unitWalkClass.getNodeSignature
            # node = {...}
            def nodeMatch(graphNode, patternNode)->bool:
                return nodeSigMatch(getNodeSignature(patternNode), getNodeSignature(graphNode))
            return nodeMatch
        
        @staticmethod
        # for networkx multiGraph matching, the matching is based on their sigature
        # if each edge in pattern match any edge in graph (allowing repeatedly match the same edge in graph), then return true, or return false
        def edgeMatchHelper(unitWalkClass:type[Unitwalk], edgeSigMatch:tuple[lambda:None])->tuple[lambda:None]:
            getEdgeSignature = unitWalkClass.getEdgeSignature
            # edges = {0: {...}, 1:{...}}
            def edgeMatch(graphEdges, patternEdges)->bool:
                for patternEdge in patternEdges.values():
                    patternEdgeSig = getEdgeSignature(patternEdge)
                    found = False
                    for graphEdge in graphEdges.values():
                        if edgeSigMatch(patternEdgeSig, getEdgeSignature(graphEdge)):
                            found = True
                            break
                    if not(found):
                        return False
                return True
            return edgeMatch
    
    # oRel is dictionary
    # e.g, oRels = {'root': {'parents': (), 'children': (0, 1, 2)}, 0: {'parents': (), 'children': (3, 4)}, 1: {'parents': (), 'children': (3, 4)}, 2: {'parents': (), 'children': (3, 4)}, 3: {'parents': (0, 1, 2), 'children': ()}, 4: {'parents': (0, 1, 2), 'children': ()}}
    # match(patternWalk:Unitwalk,logWalk:Unitwalk)->bool
    def __init__(self, unitWalkClass:type[Unitwalk], edgeFilePath:str, nodeFilePath:str, \
                 oRels:dict=None, brotherAgeDiffLimit=float('inf'), match=MatchUtility.defaultMatch, \
                 debug=False):
        self.__unitWalkClass = unitWalkClass
        self.__name = None
        self.__level = None
        # handle nodes
        self.__nodeTable = {}  # original id -> node
        with open(nodeFilePath) as nodeFile:
            for line in nodeFile:
                if not(line.startswith("#")):
                    node = json.loads(line)["node"]  # dict
                    self.__nodeTable[node["id"]] = node
        
        self.__nodeIndexTable = {}  # give node index as new id in pattern, original id -> index
        for index,(nodeId,node) in enumerate(self.__nodeTable.items()):
            self.__nodeIndexTable[nodeId] = index
        
        #print(f'{self.__nodeIndexTable}')
        
        # handle edges
        self.__unitwalks = []
        with open(edgeFilePath) as edgeFile:
            for line in edgeFile:
                if not(line.startswith("#")):
                    edge = json.loads(line)["edge"]  # dict
                    unitwalk = unitWalkClass(self.__nodeTable[edge["start"]["id"]],self.__nodeTable[edge["end"]["id"]], edge)
                    self.__unitwalks.append(unitwalk)
        self.__unitwalks.sort(key=(lambda u: (u.getEdgeTime()[0],u.getEdgeTime()[1])))
        self.__unitwalks = np.array(self.__unitwalks)
        
        #for unitwalk in self.__unitwalks:
        #    print(unitwalk)
        
        # handle order relation
        if oRels is None:
            self.__oRels = self.__constructOrelsByEdgeTimestamp(brotherAgeDiffLimit)
        else:
            self.__oRels = oRels
        
        # for match function
        self.__match = match
        
        # for experiment statistics and debugging
        self.__patternWalkMatchTimes = np.zeros(self.getNumUnitwalks(), dtype=int)
        self.__patternWalkGroundTruth = np.zeros(self.getNumUnitwalks(), dtype=int)
        self.__debug = debug
        if self.__debug:
            self.matchAll = self.__matchAll_DebugMode
            self.__patternWalkMatchWalks = np.empty(self.getNumUnitwalks(),dtype=list)
            for index in range(len(self.__patternWalkMatchWalks)):
                self.__patternWalkMatchWalks[index] = collections.deque()
        else:
            self.matchAll = self.__matchAll_NormalMode
        return
    
    def __constructOrelsByEdgeTimestamp(self, brotherAgeDiffLimit:float):
        oRels = {}
        privousUnitwalkIndexGroup = []
        currentUnitwalkIndexGroup = ['root']
        currentTime = -1
        logicalOrder = 0
        for index,unitwalk in enumerate(self.__unitwalks):
            if unitwalk.getEdgeTime()[0] > currentTime:
                parents = tuple(privousUnitwalkIndexGroup)
                for unitwalkIndex in currentUnitwalkIndexGroup:
                    oRels[unitwalkIndex] = {}
                    oRels[unitwalkIndex]['parents'] = parents
                    oRels[unitwalkIndex]['order'] = logicalOrder
                children = tuple(currentUnitwalkIndexGroup)
                for unitwalkIndex in privousUnitwalkIndexGroup:
                    oRels[unitwalkIndex]['children'] = children
                    oRels[unitwalkIndex]['brotherAgeDiffLimit'] = brotherAgeDiffLimit
                    
                privousUnitwalkIndexGroup = currentUnitwalkIndexGroup
                currentUnitwalkIndexGroup = []
                currentTime = unitwalk.getEdgeTime()[0]
                logicalOrder += 1
            currentUnitwalkIndexGroup.append(index)
        
        if len(currentUnitwalkIndexGroup) > 0:
            parents = tuple(privousUnitwalkIndexGroup)
            for unitwalkIndex in currentUnitwalkIndexGroup:
                oRels[unitwalkIndex] = {}
                oRels[unitwalkIndex]['parents'] = parents
                oRels[unitwalkIndex]['order'] = logicalOrder
                oRels[unitwalkIndex]['children'] = tuple()
                oRels[unitwalkIndex]['brotherAgeDiffLimit'] = brotherAgeDiffLimit
                
            children = tuple(currentUnitwalkIndexGroup)
            for unitwalkIndex in privousUnitwalkIndexGroup:
                oRels[unitwalkIndex]['children'] = children
                oRels[unitwalkIndex]['brotherAgeDiffLimit'] = brotherAgeDiffLimit
        
        #for unitwalkIndex in oRels['root']['children']:
        #    oRels[unitwalkIndex]['parents'] = tuple()
        #oRels['root'].pop('brotherAgeDiffLimit')
        
        return oRels
    
    def __matchGroundTruth(self,patternWalk:Unitwalk,logWalk:Unitwalk)->bool:
        for part in [Unitwalk.Part.EDGE,Unitwalk.Part.NODE_FROM,Unitwalk.Part.NODE_TO]:
            if patternWalk.getId(part) != logWalk.getId(part):
                return False
        return True
    
    def matchAll(self,logWalk:Unitwalk)->list:
        raise Exception(f'The default matchAll function is not implemented, please overwrite a new function')
    
    def checkIsMatchAll(self,logWalk:Unitwalk)->bool:
        for i,patternWalk in enumerate(self.__unitwalks):
            if self.__match(patternWalk,logWalk):
                return True
        return False
    
    def _Pattern__matchAll_NormalMode(self,logWalk:Unitwalk)->list:
        matchedPatternWalkIndexes = []
        for i,patternWalk in enumerate(self.__unitwalks):
            if self.__match(patternWalk,logWalk):
                matchedPatternWalkIndexes.append(i)
                self.__patternWalkMatchTimes[i] += 1
        return matchedPatternWalkIndexes
    
    def _Pattern__matchAll_DebugMode(self,logWalk:Unitwalk)->list:
        matchedPatternWalkIndexes = self.__matchAll_NormalMode(logWalk)
        for index in matchedPatternWalkIndexes:
            self.__patternWalkMatchWalks[index].append(logWalk)
        for i,patternWalk in enumerate(self.__unitwalks):
            if self.__matchGroundTruth(patternWalk,logWalk):
                #print(f'id: {i} patternWalk is found in log')
                #print(f'{logWalk}')
                self.__patternWalkGroundTruth[i] += 1
        return matchedPatternWalkIndexes
    
    def getName(self):
        return self.__name
    def setName(self,name):
        self.__name = name
    def getLevel(self):
        return self.__level
    def setLevel(self,level):
        self.__level = level
    def getNumNodes(self):
        return len(self.__nodeTable)
    def getNumUnitwalks(self):
        return len(self.__unitwalks)
    def getORelsInCopy(self)->dict:
        return self.__oRels.copy()
    def getNodeIndexByPatternWalkIndex(self, part:Unitwalk.Part, patternWalkIndex:int):
        return self.__nodeIndexTable[(self.__unitwalks[patternWalkIndex].getId(part))]
    def getORelParentsByPatternWalkIndex(self, patternWalkIndex:int):
        return self.__oRels[patternWalkIndex]['parents']
    def getORelChildrenByPatternWalkIndex(self, patternWalkIndex:int):
        return self.__oRels[patternWalkIndex]['children']
    def getORelBrotherAgeDiffLimitByPatternWalkIndex(self, patternWalkIndex:int):
        return self.__oRels[patternWalkIndex]['brotherAgeDiffLimit']
    def calculateDuration(self)->float:
        return self.__unitwalks[-1].getEdgeTime()[1] - self.__unitwalks[0].getEdgeTime()[0]
    
    # for experiment statistics and debugging
    def getPatternWalkMatchTimes(self)->list:
        return self.__patternWalkMatchTimes.tolist()
    def getPatternWalkGroundTruth(self)->list:
        return self.__patternWalkGroundTruth.tolist()
    def getPatternWalkMatchWalks(self)->list:
        return self.__patternWalkMatchWalks.tolist()
    
    # for presentation
    def toPyvisNetwork(self)->Network:
        network = Network(height='100%',width='60%',directed=True,notebook=False,layout=True)
        for unitwalkId,unitwalk in enumerate(self.__unitwalks):
            nodeFromId = self.__nodeIndexTable[unitwalk.getId(Unitwalk.Part.NODE_FROM)]
            nodeToId = self.__nodeIndexTable[unitwalk.getId(Unitwalk.Part.NODE_TO)]
            
            network.add_node(nodeFromId , \
                             title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_FROM,detail=True),withKey=True), \
                             label = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_FROM,detail=False),withKey=False), \
                             shape = 'box', \
                             physics = True \
                            )
            
            network.add_node(nodeToId , \
                             title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_TO,detail=True),withKey=True), \
                             label = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.NODE_TO,detail=False),withKey=False), \
                             shape = 'box', \
                             physics = True \
                            )
            
            edgeConciseAttr = unitwalk.getConciseAttr(Unitwalk.Part.EDGE,detail=False)
            edgeConciseAttr['order'] = self.__oRels[unitwalkId]['order']
            network.add_edge(nodeFromId, \
                             nodeToId, \
                             title = NodeEdgeUtility.conciseAttrToLines(unitwalk.getConciseAttr(Unitwalk.Part.EDGE,detail=True),withKey=True), \
                             label = NodeEdgeUtility.conciseAttrToLines(edgeConciseAttr,withKey=False), \
                             physics = True \
                            )
        
        for originalNodeId,node in self.__nodeTable.items():
            nodeId = self.__nodeIndexTable[originalNodeId]
            network.add_node(nodeId , \
                             title = NodeEdgeUtility.conciseAttrToLines(self.__unitWalkClass.getNodeConciseAttr(node,detail=True),withKey=True), \
                             label = NodeEdgeUtility.conciseAttrToLines(self.__unitWalkClass.getNodeConciseAttr(node,detail=False),withKey=False), \
                             shape = 'box', \
                             physics = True \
                            )
        return network
    
    def toNetworkxGraph(self):
        graph = nx.MultiDiGraph()
        for unitwalk in self.__unitwalks:
            fromNode = unitwalk.getRawdataInCopy(Unitwalk.Part.NODE_FROM)
            edge = unitwalk.getRawdataInCopy(Unitwalk.Part.EDGE)
            toNode = unitwalk.getRawdataInCopy(Unitwalk.Part.NODE_TO)
            
            fromNodeId = self.__nodeIndexTable[NodeEdgeUtility.getId(fromNode)]
            toNodeId = self.__nodeIndexTable[NodeEdgeUtility.getId(toNode)]
            if fromNodeId not in graph:
                graph.add_nodes_from([(fromNodeId,fromNode)])
            if toNodeId not in graph:
                graph.add_nodes_from([(toNodeId,toNode)])
            graph.add_edges_from([(fromNodeId,toNodeId,edge)])
        
        for originalNodeId,node in self.__nodeTable.items():
            nodeId = self.__nodeIndexTable[originalNodeId]
            if nodeId not in graph:
                graph.add_nodes_from([(nodeId,node)])
        return graph
    
    def __str__(self):
        buffer = []
        buffer.append(f'name: {self.getName()}\n')
        buffer.append(f'level: {self.getLevel()}\n')
        buffer.append(f'unitwalks:\n')
        for unitwalkId,unitwalk in enumerate(self.__unitwalks):
            buffer.append(f'\tid: {unitwalkId}:\n')
            buffer.append(f'\t{unitwalk.getConciseAttr(Unitwalk.Part.NODE_FROM,detail=False)} id: {self.getNodeIndexByPatternWalkIndex(Unitwalk.Part.NODE_FROM,unitwalkId)}\n')
            buffer.append(f'\t{unitwalk.getConciseAttr(Unitwalk.Part.EDGE,detail=False)}\n')
            buffer.append(f'\t{unitwalk.getConciseAttr(Unitwalk.Part.NODE_TO,detail=False)} id: {self.getNodeIndexByPatternWalkIndex(Unitwalk.Part.NODE_TO,unitwalkId)}\n')
        return ''.join(buffer)
    
    def __setstate__(self, state):
        self.__dict__.update(state)
        self.__name = None
        self.__level = None
        return

def __testing():
    def nodeAttrsTransformer(nodeAttrs):
        newNodeAttrs = {}
        if nodeAttrs['color'] == 'red':
            newNodeAttrs['color'] = 'red2'
        else:
            newNodeAttrs['color'] = nodeAttrs['color']
        return newNodeAttrs
    def edgeAttrsTransformer(edgeAttrsList):
        newEdgeAttrs = {'w':0}
        for edgeAttrs in edgeAttrsList:
            newEdgeAttrs['w'] = newEdgeAttrs['w'] + edgeAttrs['w']
        return newEdgeAttrs

    graph = nx.MultiDiGraph()
    graph.add_node(1, color="red")
    graph.add_node(2, color="yellow")
    graph.add_node(3, color="green")

    graph.add_edge(1,2, w=1)
    graph.add_edge(1,2, w=2)
    graph.add_edge(2,1, w=3)
    graph.add_edge(2,3, w=4)
    graph.add_edge(3,3, w=5)
    
    print("===== original =====")
    print(graph.nodes(data=True))
    print(graph.edges(data=True))
    newGraph = TransformUtility.transformNetworkxGraphForNeuroMatch(graph, toUndirected=False,
                                                                  nodeAttrsTransformer=nodeAttrsTransformer, edgeAttrsTransformer=edgeAttrsTransformer)
    print("===== undirected =====")
    print(newGraph.nodes(data=True))
    print(newGraph.edges(data=True))
    newGraph = TransformUtility.transformNetworkxGraphForNeuroMatch(graph, toUndirected=True,
                                                                  nodeAttrsTransformer=nodeAttrsTransformer, edgeAttrsTransformer=edgeAttrsTransformer)
    print("===== directed =====")
    print(newGraph.nodes(data=True))
    print(newGraph.edges(data=True))
#__testing()

def __testing2():
    logFilePath = '/home/gccbg00382/APT/Experiement_Datasets/Testing/raw.json'
    #logFilePath = 'D:\\APT\\Experiement_Datasets\\Testing\\raw.json'
    
    #graph = TransformUtility.transformTraceToPyvisNetwork(TransformUtility.createReadUnitwalksGenerator(unitWalkClass=SPADE_Unitwalk,logFilePath=logFilePath))
    #print(f'{len(graph.nodes)} {len(graph.edges)}')
    graph = TransformUtility.transformTraceToNetworkxGraph(TransformUtility.createReadUnitwalksGenerator(logFilePath=logFilePath,withTqdm=True))
    print(f'{len(graph.nodes)} {len(graph.edges)}')
    newGraph = TransformUtility.transformNetworkxGraphForNeuroMatch(graph, toUndirected=True)
    print(f'{len(newGraph.nodes)} {len(newGraph.edges)}')
    newGraph2 = TransformUtility.transformNetworkxGraphForNeuroMatch(graph, toUndirected=False)
    print(f'{len(newGraph2.nodes)} {len(newGraph2.edges)}')
    print(newGraph.nodes["5"])
    print(newGraph["5"]["5"])
#__testing2()