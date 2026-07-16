#!/usr/bin/env python3
"""Build the evidence-backed Clio systems monograph as a polished PDF."""

import csv
import html
import re
import textwrap
from pathlib import Path

from reportlab.graphics.charts.lineplots import LinePlot
from reportlab.graphics.shapes import Drawing, Rect, String
from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import BaseDocTemplate, Frame, Image, PageBreak, PageTemplate, Paragraph, Preformatted, Spacer, Table, TableStyle
from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[3]
OUTPUT = ROOT / "output/pdf/clio-vqpu-research-paper.pdf"
ACCENT = colors.HexColor("#667d00")
INK = colors.HexColor("#18201c")
MUTED = colors.HexColor("#59645e")
LINE = colors.HexColor("#c8cfca")


def ascii_text(value):
    return value.replace("—", "-").replace("–", "-").replace("×", "x").replace("π", "pi").replace("→", "->").replace("↓", "v").replace("≥", ">=").replace("≤", "<=").replace("…", "...").replace("“", '"').replace("”", '"').replace("’", "'")


def code_text(value):
    wrapped=[]
    for line in ascii_text(value).splitlines():
        indent=re.match(r"\s*",line).group(0)
        wrapped.extend(textwrap.wrap(line,width=138,subsequent_indent=indent+"  ",replace_whitespace=False,drop_whitespace=False,break_long_words=True,break_on_hyphens=False) or [""])
    return "\n".join(wrapped)


class PaperDoc(BaseDocTemplate):
    def __init__(self, filename):
        super().__init__(filename, pagesize=A4, leftMargin=22*mm, rightMargin=20*mm, topMargin=22*mm, bottomMargin=20*mm, title="Clio: Design, Implementation, and Evaluation of a Programmable Virtual Quantum Processing Unit", author="Advaith Praveen - Archeum Studios")
        frame = Frame(self.leftMargin, self.bottomMargin, self.width, self.height, id="normal")
        self.addPageTemplates(PageTemplate(id="main", frames=frame, onPage=page_decor))


def page_decor(canvas, doc):
    if doc.page == 1: return
    canvas.saveState(); canvas.setStrokeColor(LINE); canvas.line(22*mm, 14*mm, 190*mm, 14*mm)
    canvas.setFont("Helvetica", 7); canvas.setFillColor(MUTED); canvas.drawString(22*mm, 9*mm, "CLIO VQPU · ARCHEUM STUDIOS · RESEARCH MONOGRAPH")
    canvas.drawRightString(190*mm, 9*mm, str(doc.page)); canvas.restoreState()


styles = getSampleStyleSheet()
styles.add(ParagraphStyle(name="PaperTitle", parent=styles["Title"], fontName="Helvetica-Bold", fontSize=27, leading=31, textColor=INK, alignment=TA_CENTER, spaceAfter=8*mm))
styles.add(ParagraphStyle(name="Subtitle", parent=styles["Normal"], fontName="Helvetica", fontSize=12, leading=17, textColor=MUTED, alignment=TA_CENTER))
styles.add(ParagraphStyle(name="H1x", parent=styles["Heading1"], fontName="Helvetica-Bold", fontSize=20, leading=24, textColor=INK, spaceBefore=7*mm, spaceAfter=4*mm))
styles.add(ParagraphStyle(name="H2x", parent=styles["Heading2"], fontName="Helvetica-Bold", fontSize=13, leading=16, textColor=ACCENT, spaceBefore=5*mm, spaceAfter=2*mm))
styles.add(ParagraphStyle(name="Bodyx", parent=styles["BodyText"], fontName="Times-Roman", fontSize=9.3, leading=13.2, textColor=INK, spaceAfter=2.6*mm, alignment=4))
styles.add(ParagraphStyle(name="Bulletx", parent=styles["Bodyx"], leftIndent=6*mm, firstLineIndent=-3*mm))
styles.add(ParagraphStyle(name="Caption", parent=styles["Bodyx"], fontName="Helvetica-Oblique", fontSize=7.5, leading=10, textColor=MUTED, alignment=TA_CENTER, spaceBefore=2*mm, spaceAfter=5*mm))
styles.add(ParagraphStyle(name="Codex", fontName="Courier", fontSize=5.4, leading=6.6, textColor=INK, backColor=colors.HexColor("#f1f4f2"), borderPadding=4, spaceAfter=3*mm))


def markdown_flow(path):
    lines = path.read_text(encoding="utf-8").splitlines(); flow=[]; paragraph=[]; code=[]; in_code=False
    def flush():
        if paragraph:
            text=" ".join(x.strip() for x in paragraph); flow.append(Paragraph(inline(text), styles["Bodyx"])); paragraph.clear()
    for line in lines:
        if line.startswith("```"):
            if in_code: flow.append(Preformatted(code_text("\n".join(code)), styles["Codex"])); code.clear()
            else: flush()
            in_code=not in_code; continue
        if in_code: code.append(line); continue
        if line.startswith("# "): flush(); flow.extend([PageBreak(), Paragraph(inline(line[2:]), styles["H1x"])]); continue
        if line.startswith("## "): flush(); flow.append(Paragraph(inline(line[3:]), styles["H2x"])); continue
        if line.startswith("- "): flush(); flow.append(Paragraph("• "+inline(line[2:]), styles["Bulletx"])); continue
        if not line.strip(): flush(); continue
        if line.startswith("|"): flush(); continue
        paragraph.append(line)
    flush(); return flow


def inline(value):
    safe=html.escape(ascii_text(value)); safe=re.sub(r"`([^`]+)`", r"<font name='Courier'>\1</font>", safe); safe=re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", safe); return safe


def architecture_drawing():
    d=Drawing(470,250); labels=["Clio source", "Parser + assembler", "Resource admission", "Processor runtime", "Clio Engine", "Trace + result", "Validation + replay"]
    for i,label in enumerate(labels):
        x=15+(i%4)*113; y=165-(i//4)*92; d.add(Rect(x,y,96,45,strokeColor=ACCENT,fillColor=colors.HexColor("#f5f7ed"),rx=4,ry=4)); d.add(String(x+48,y+22,label,textAnchor="middle",fontName="Helvetica",fontSize=7,fillColor=INK))
        if i and i%4: d.add(String(x-8,y+20,"->",fontName="Helvetica-Bold",fontSize=9,fillColor=MUTED))
    d.add(String(235,230,"Clio source-to-evidence architecture",textAnchor="middle",fontName="Helvetica-Bold",fontSize=12,fillColor=INK)); return d


def chart(family, x_label):
    rows=list(csv.DictReader((ROOT/"research/benchmarks/final/processed/benchmark-summary.csv").open(encoding="utf-8"))); points=[(float(r["parameter"]),float(r["median_ns"])/1e6) for r in rows if r["family"]==family]
    d=Drawing(470,250); plot=LinePlot(); plot.x=55; plot.y=45; plot.width=380; plot.height=165; plot.data=[points]; plot.lines[0].strokeColor=ACCENT; plot.lines[0].strokeWidth=2; plot.lines[0].symbol=None; plot.xValueAxis.valueMin=0; plot.xValueAxis.valueMax=max(x for x,_ in points); plot.yValueAxis.valueMin=0; plot.yValueAxis.valueMax=max(y for _,y in points)*1.1; plot.xValueAxis.labelTextFormat="%g"; plot.yValueAxis.labelTextFormat="%.2g"; d.add(plot); d.add(String(245,228,f"Median runtime versus {x_label}",textAnchor="middle",fontName="Helvetica-Bold",fontSize=12,fillColor=INK)); d.add(String(245,18,x_label,textAnchor="middle",fontName="Helvetica",fontSize=8,fillColor=MUTED)); d.add(String(12,130,"runtime (ms)",fontName="Helvetica",fontSize=8,fillColor=MUTED)); return d


def appendix_file(path):
    relative=path.relative_to(ROOT); text=code_text(path.read_text(encoding="utf-8",errors="replace")); return [PageBreak(),Paragraph(f"Appendix: {html.escape(str(relative))}",styles["H1x"]),Paragraph("The following checked-in artifact is reproduced for semantic and implementation audit.",styles["Bodyx"]),Preformatted(text,styles["Codex"])]


def build():
    OUTPUT.parent.mkdir(parents=True,exist_ok=True); story=[Spacer(1,26*mm),Paragraph("CLIO",styles["PaperTitle"]),Paragraph("Design, Implementation, and Evaluation of a<br/>Programmable Virtual Quantum Processing Unit",styles["Subtitle"]),Spacer(1,16*mm),Paragraph("Advaith Praveen",styles["Subtitle"]),Paragraph("Archeum Studios",styles["Subtitle"]),Spacer(1,16*mm),Paragraph("Definitive research artifact · Built from the verified repository implementation and retained evidence",styles["Caption"]),PageBreak(),Paragraph("Artifact map",styles["H1x"])]
    chapter_paths=sorted((ROOT/"research/paper/chapters").glob("*.md"));
    for path in chapter_paths: story.append(Paragraph(html.escape(path.stem.replace("-"," ").title()),styles["Bodyx"]))
    story.extend([Spacer(1,5*mm),architecture_drawing(),Paragraph("Figure 1. The owned Clio pipeline from source through evidence.",styles["Caption"])] )
    for path in chapter_paths: story.extend(markdown_flow(path))
    studio_image=ROOT/"research/figures/generated/clio-studio-real-execution.png"
    story.extend([PageBreak(),Paragraph("Product and Evaluation Figures",styles["H1x"]),Image(str(studio_image),width=170*mm,height=95.625*mm),Paragraph("Clio Studio displaying a real 1,024-shot Bell execution and a separate seven-step bounded inspection trace.",styles["Caption"]),Paragraph("All plots below are generated from the retained ten-repetition release-build dataset. Exact records, environment, commands, and checksums accompany the repository.",styles["Bodyx"])])
    for family,label in [("qubits","allocated qubits"),("depth","circuit depth"),("shots","shots"),("trace","trace level"),("abstraction","execution path")]: story.extend([chart(family,label),Paragraph(f"Generated final-protocol median for {label}; source: benchmark-summary.csv.",styles["Caption"])])
    story.extend([PageBreak(),Paragraph("Reproducibility Tables",styles["H1x"])] )
    for csv_path in [ROOT/"research/benchmarks/external/qiskit-comparison.csv",ROOT/"research/benchmarks/processed/resource-trace-results.csv"]:
        rows=list(csv.reader(csv_path.open(encoding="utf-8"))); shown=[[ascii_text(cell)[:24] for cell in row] for row in rows]; table=Table(shown,repeatRows=1,hAlign="LEFT"); table.setStyle(TableStyle([("FONT",(0,0),(-1,-1),"Helvetica",5.5),("BACKGROUND",(0,0),(-1,0),colors.HexColor("#e9eee4")),("GRID",(0,0),(-1,-1),.25,LINE),("VALIGN",(0,0),(-1,-1),"TOP"),("LEFTPADDING",(0,0),(-1,-1),2),("RIGHTPADDING",(0,0),(-1,-1),2)])); story.extend([Paragraph(html.escape(str(csv_path.relative_to(ROOT))),styles["H2x"]),table,Spacer(1,5*mm)])
    appendix_paths=list(sorted((ROOT/"spec").glob("**/*.md")))+list(sorted((ROOT/"docs").glob("**/*.md")))
    source_names=["clio-engine","clio-runtime","clio-assembler","clio-resource","clio-trace","clio-validation","clio-sdk","clio-replay","clio-studio"]
    appendix_paths += [ROOT/f"crates/{name}/src/lib.rs" for name in source_names if (ROOT/f"crates/{name}/src/lib.rs").exists()]
    appendix_paths += [ROOT/"crates/clio-studio/src/main.rs",ROOT/"crates/clio-cli/src/main.rs"]
    for path in appendix_paths: story.extend(appendix_file(path))
    PaperDoc(str(OUTPUT)).build(story)
    pages=len(PdfReader(str(OUTPUT)).pages); print(f"wrote {OUTPUT} ({pages} pages)")
    if not 60 <= pages <= 110: raise SystemExit(f"paper page count {pages} outside required substantial range")


if __name__ == "__main__": build()
