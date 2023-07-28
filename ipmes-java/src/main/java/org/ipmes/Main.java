package org.ipmes;

import java.io.FileReader;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;

public class Main {
    public static void main(String[] args) throws Exception {
        // parse data
        String ttpPrefix = args[0];
        boolean useRegex = false;
        String orelsFile;
        if (ttpPrefix.endsWith("regex")) {
            useRegex = true;
            orelsFile = ttpPrefix.substring(0, ttpPrefix.length() - 5) + "oRels.json";
        } else {
            orelsFile = ttpPrefix + "_oRels.json";
        }
        PatternGraph pattern = PatternGraph.parse(new FileReader(ttpPrefix + "_node.json"), new FileReader(ttpPrefix + "_edge.json")).get();
        DependencyGraph dep = DependencyGraph.parse(new FileReader(orelsFile)).get();

        // Decomposition
        Decomposition d = new Decomposition(dep, pattern);
        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern, dep, d.decompose());
        gen.setUseRegex(useRegex);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        System.out.println(appStr);
        SiddhiManager siddhiManager = new SiddhiManager();
        SiddhiAppRuntime runtime = siddhiManager.createSiddhiAppRuntime(appStr);
    }
}