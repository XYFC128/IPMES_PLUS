package org.ipmes.decomposition.matcher;

import io.siddhi.core.SiddhiAppRuntime;
import io.siddhi.core.SiddhiManager;
import io.siddhi.core.stream.input.InputHandler;
import org.ipmes.EventEdge;
import org.ipmes.decomposition.TCMatcher;
import org.ipmes.decomposition.TCQuery;
import org.ipmes.event.EventSorter;
import org.ipmes.join.Join;
import org.ipmes.pattern.Pattern;
import org.ipmes.siddhi.TCSiddhiAppGenerator;
import org.ipmes.siddhi.TCQueryOutputCallback;
import java.util.ArrayList;

public class CEPMatcher implements TCMatcher {
    Join join;
    SiddhiAppRuntime runtime;
    InputHandler inputHandler;
    EventSorter sorter;

    public CEPMatcher(Pattern pattern, ArrayList<TCQuery> tcQueries, long windowSize, Join join) {
        this.join = join;

        TCSiddhiAppGenerator gen = new TCSiddhiAppGenerator(pattern.patternGraph, pattern.temporalRelation, tcQueries, windowSize);
        gen.setUseRegex(pattern.useRegex);

        // Generate CEP app and runtime
        String appStr = gen.generate();
        SiddhiManager siddhiManager = new SiddhiManager();

        this.runtime = siddhiManager.createSiddhiAppRuntime(appStr);

        this.inputHandler = runtime.getInputHandler("InputStream");

        for (TCQuery q : tcQueries) {
            runtime.addCallback(
                    String.format("TC%dOutput", q.getId()),
                    new TCQueryOutputCallback(q, pattern.patternGraph, join));
        }

        this.sorter = new EventSorter(tcQueries, pattern.useRegex);
        runtime.start();
    }

    public void sendAll(ArrayList<EventEdge> events) throws InterruptedException {
        if (events.isEmpty())
            return;
        EventEdge first = events.iterator().next();
        ArrayList<Object[]> sorted = sorter.rearrangeToEventData(events);
        for (Object[] data : sorted)
            inputHandler.send(first.timestamp, data);
    }

    public int getPoolSize() {
        return 0;
    }

    public ArrayList<long[]> getTriggerCounts() {
        return new ArrayList<>();
    }
}
