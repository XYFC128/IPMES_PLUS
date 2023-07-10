package com.pattern_recognition;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;

import org.apache.flink.api.common.functions.FlatMapFunction;
import org.apache.flink.api.common.functions.MapFunction;
import org.apache.flink.api.java.DataSet;
import org.apache.flink.api.java.ExecutionEnvironment;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.api.java.tuple.Tuple3;
import org.apache.flink.api.java.utils.MultipleParameterTool;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.node.ObjectNode;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.util.Collector;
import org.apache.flink.util.Preconditions;
import org.json.*;
import org.w3c.dom.events.Event;

import javafx.scene.control.Alert;

import java.util.List;
import java.util.Map;

import org.apache.flink.cep.CEP;
import org.apache.flink.cep.CEP.*;
import org.apache.flink.cep.PatternSelectFunction;
import org.apache.flink.cep.functions.PatternProcessFunction;
import org.apache.flink.cep.PatternStream;
import org.apache.flink.cep.pattern.Pattern;
import org.apache.flink.cep.pattern.conditions.SimpleCondition;

import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;

public class test {

    // define structure of Edge
    public static class Edge {
        public float node_time[] = new float[2];
        public String node_source[] = new String[2];
        public String node_type[] = new String[2];
        public String node_subtype[] = new String[2];
        public String edge_operation;
        public int init;

        public Edge() {
            this.init = 1;
        }

        public static Edge set_edge(Edge edge, float node_time[], String node_source[],
                String node_type[], String node_subtype[], String edge_operation) {
            edge.edge_operation = edge_operation;
            for (int i = 0; i < 2; i++) {
                edge.node_time[i] = node_time[i];
                edge.node_source[i] = node_source[i];
                edge.node_type[i] = node_type[i];
                edge.node_subtype[i] = node_subtype[i];
            }
            return edge;
        }
    }

    public static Edge createEdgeFromRawData(JSONObject rawData) {
        JSONObject node[] = new JSONObject[2];
        node[0] = rawData.getJSONObject("m");
        node[1] = rawData.getJSONObject("n");
        JSONObject edge = rawData.getJSONObject("r");
        String node_source[] = new String[2];
        String node_type[] = new String[2];
        String node_subtype[] = new String[2];
        float node_time[] = new float[2];
        for (int i = 0; i < 2; i++) {
            node_source[i] = node[i].getJSONObject("properties").getString("source");
            node_type[i] = node[i].getJSONObject("properties").getString("type");
            if (node_type[i].equals("Process")) {
                node_subtype[i] = null;
                try {
                    node_time[i] = node[i].getJSONObject("properties").getFloat("seen time");
                } catch (Exception e) {
                    node_time[i] = node[i].getJSONObject("properties").getFloat("start time");
                }
            } else {
                node_subtype[i] = node[i].getJSONObject("properties").getString("subtype");
                node_time[i] = 0;
            }
        }
        String edge_operation = edge.getJSONObject("properties").getString("operation");
        Edge EDGE = new Edge();
        EDGE = Edge.set_edge(EDGE, node_time, node_source, node_type, node_subtype, edge_operation);
        return EDGE;
    }

    public static void main(String[] args) throws Exception {

        // set up the execution environment
        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
        // read and parse JSON
        String file_path = "/Users/datou/repos/CITI/graphs/test.json";
        String content = new String(Files.readAllBytes(Paths.get(file_path)));
        String lines[] = content.split("\n");
        List<Edge> edges = new ArrayList<Edge>();

        for (String line : lines) {
            JSONObject rawData = new JSONObject(line);
            Edge edge = createEdgeFromRawData(rawData);
            edges.add(edge);
        }
        DataStream<Edge> input = env.fromCollection(edges);
        input.map(new MapFunction<Edge, String>() {
            @Override
            public String map(Edge edge) throws Exception {
                return edge.edge_operation;
            }
        }).print();
        Pattern<Edge, ?> pattern = Pattern.<Edge>begin("start")
                .where(SimpleCondition.of(edge -> edge.node_source[0].equals("syscall")));
        PatternStream<Edge> patternStream = CEP.pattern(input, pattern);
        DataStream<String> result = patternStream.select(new PatternSelectFunction<test.Edge, String>() {
            @Override
            public String select(Map<String, List<Edge>> p) throws Exception {
                String strResult = "a";
                for (int i = 0; i < p.get("start").size(); i++) {
                    strResult += p.get("start").get(i).edge_operation + " ";
                }
                return "strResult";
            }
        });

        result.print();
        env.execute("Stream");
    }
}
