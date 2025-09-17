/**
 * Security Framework Model
 * Represents a security framework with its controls and requirements
 */
export class SecurityFramework {
  constructor(id, name, version, description) {
    this.id = id;
    this.name = name;
    this.version = version;
    this.description = description;
    this.domains = new Map();
    this.controls = new Map();
  }

  addDomain(domain) {
    this.domains.set(domain.id, domain);
  }

  addControl(control) {
    this.controls.set(control.id, control);
    
    // Associate control with domain if specified
    if (control.domainId && this.domains.has(control.domainId)) {
      this.domains.get(control.domainId).addControl(control);
    }
  }

  getControl(controlId) {
    return this.controls.get(controlId);
  }

  getAllControls() {
    return Array.from(this.controls.values());
  }

  getDomain(domainId) {
    return this.domains.get(domainId);
  }

  getAllDomains() {
    return Array.from(this.domains.values());
  }
}

/**
 * Security Domain Model
 * Represents a domain or category within a security framework
 */
export class SecurityDomain {
  constructor(id, name, description, frameworkId) {
    this.id = id;
    this.name = name;
    this.description = description;
    this.frameworkId = frameworkId;
    this.controls = new Map();
  }

  addControl(control) {
    this.controls.set(control.id, control);
  }

  getControls() {
    return Array.from(this.controls.values());
  }
}

/**
 * Security Control Model
 * Represents an individual security control or requirement
 */
export class SecurityControl {
  constructor(id, title, description, frameworkId, domainId = null) {
    this.id = id;
    this.title = title;
    this.description = description;
    this.frameworkId = frameworkId;
    this.domainId = domainId;
    this.requirements = [];
    this.implementationGuidance = '';
    this.testingProcedures = '';
    this.riskLevel = 'medium'; // low, medium, high, critical
    this.controlType = 'procedural'; // technical, procedural, physical
    this.tags = [];
    this.mappings = new Map(); // Mappings to other framework controls
  }

  addRequirement(requirement) {
    this.requirements.push(requirement);
  }

  addMapping(frameworkId, controlId, mappingType = 'equivalent') {
    if (!this.mappings.has(frameworkId)) {
      this.mappings.set(frameworkId, []);
    }
    this.mappings.get(frameworkId).push({
      controlId,
      mappingType, // equivalent, partial, related, parent, child
      confidence: 1.0 // 0.0 to 1.0 confidence score
    });
  }

  getMappings(frameworkId = null) {
    if (frameworkId) {
      return this.mappings.get(frameworkId) || [];
    }
    return this.mappings;
  }

  addTag(tag) {
    if (!this.tags.includes(tag)) {
      this.tags.push(tag);
    }
  }
}

/**
 * Control Mapping Model
 * Represents a mapping relationship between controls across frameworks
 */
export class ControlMapping {
  constructor(sourceFramework, sourceControl, targetFramework, targetControl, mappingType = 'equivalent') {
    this.sourceFramework = sourceFramework;
    this.sourceControl = sourceControl;
    this.targetFramework = targetFramework;
    this.targetControl = targetControl;
    this.mappingType = mappingType; // equivalent, partial, related, parent, child
    this.confidence = 1.0;
    this.notes = '';
    this.verified = false;
    this.lastUpdated = new Date();
  }

  setConfidence(confidence) {
    this.confidence = Math.max(0, Math.min(1, confidence));
  }

  addNotes(notes) {
    this.notes = notes;
  }

  verify() {
    this.verified = true;
    this.lastUpdated = new Date();
  }
}