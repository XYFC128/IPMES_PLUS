package com.pattern_recognition;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;

import org.apache.flink.api.common.functions.FlatMapFunction;
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

public class App {

    // define structure of Edge
    public static class Edge {
        public float node_time[] = new float[2];
        public String node_source[] = new String[2];
        public String node_type[] = new String[2];
        public String node_subtype[] = new String[2];
        public String edge_operation;

        public Edge(float node_time[], String node_source[], String node_type[],
                String node_subtype[], String edge_operation) {
            this.edge_operation = edge_operation;
            for (int i = 0; i < 2; i++) {
                this.node_time[i] = node_time[i];
                this.node_source[i] = node_source[i];
                this.node_type[i] = node_type[i];
                this.node_subtype[i] = node_subtype[i];
            }
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
        Edge EDGE = new Edge(node_time, node_source, node_type, node_subtype, edge_operation);
        return EDGE;
    }

    public static void main(String[] args) throws IOException {

        // set up the execution environment
        final ExecutionEnvironment env = ExecutionEnvironment.getExecutionEnvironment();
        // read and parse JSON
        String file_path = "/Users/datou/repos/CITI/graphs/12hours_mix(reduced).json";
        String content = new String(Files.readAllBytes(Paths.get(file_path)));
        String lines[] = content.split("\n");
        ArrayList<Edge> edges = new ArrayList<>();

        for (String line : lines) {
            JSONObject rawData = new JSONObject(line);
            Edge edge = createEdgeFromRawData(rawData);
            edges.add(edge);
        }
        int cnt = 1;
        for (Edge edge : edges) {
            System.out.println(cnt + " " + edge.node_source[0] + " " + edge.node_type[1]);
            cnt++;
        }
    }
}
