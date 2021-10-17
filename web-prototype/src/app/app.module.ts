import {NgModule} from '@angular/core';
import {BrowserModule} from '@angular/platform-browser';

import {AppRoutingModule} from './app-routing.module';
import {AppComponent} from './app.component';
import {LoginComponent} from './auth/login/login.component';
import {BrowserAnimationsModule} from '@angular/platform-browser/animations';
import {ServiceWorkerModule} from '@angular/service-worker';
import {environment} from '../environments/environment';
import {MatToolbarModule} from "@angular/material/toolbar";
import {NgbModule} from '@ng-bootstrap/ng-bootstrap';
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatIconModule} from "@angular/material/icon";
import {GraphQLModule} from './graphql.module';
import {HttpClientModule} from '@angular/common/http';
import {MatTableModule} from "@angular/material/table";
import {CdkTableModule} from "@angular/cdk/table";
import {APOLLO_NAMED_OPTIONS, NamedOptions} from 'apollo-angular';
import {HttpLink} from 'apollo-angular/http';
import {InMemoryCache} from '@apollo/client/core';
import {SearchApi} from "./api/search.api";
import {FormsModule, ReactiveFormsModule} from "@angular/forms";
import {AppSearchService} from "./search/search.service";
import {MatChipsModule} from '@angular/material/chips';
import {MatDividerModule} from '@angular/material/divider';
import {MatSidenavModule} from "@angular/material/sidenav";
import {SideNavService} from "./services/sidenav.service";
import {MatMenuModule} from "@angular/material/menu";
import {MatButtonModule} from "@angular/material/button";
import {SearchModule} from "./search/search.module";
import {AccountComponent} from "./auth/account/account.component";
import {DatasetModule} from "./dataset-view/dataset.module";
import {AppDatasetService} from "./dataset-view/dataset.service";
import {DatasetCreateModule} from "./dataset-create/dataset-create.module";
import {AppHeaderComponent} from "./components/app-header/app-header.component";
import {MatOptionModule} from "@angular/material/core";
import {MatAutocompleteModule} from "@angular/material/autocomplete";


const Services = [
    SearchApi,
    AppSearchService,
    AppDatasetService,
    SideNavService,
    {
        provide: APOLLO_NAMED_OPTIONS,
        useFactory(httpLink: HttpLink): NamedOptions {
            return {
                newClientName: {
                    cache: new InMemoryCache(),
                    link: httpLink.create({
                        uri: 'http://0.0.0.0:8080/graphql',
                    }),
                },
            };
        },
        deps: [HttpLink],
    }
];
const MatModules = [
    MatChipsModule,
    MatDividerModule,
    MatToolbarModule,
    MatFormFieldModule,
    MatTableModule,
    MatIconModule,
    MatSidenavModule,
    MatMenuModule,
    MatButtonModule,
    MatAutocompleteModule
]

@NgModule({
    declarations: [
        AppComponent,
        AppHeaderComponent,
        LoginComponent,
        AccountComponent,
    ],
    imports: [
        AppRoutingModule,
        DatasetModule,
        DatasetCreateModule,
        SearchModule.forRoot(),


        BrowserModule,
        BrowserAnimationsModule,
        ServiceWorkerModule.register('ngsw-worker.js', {
            enabled: environment.production,
            // Register the ServiceWorker as soon as the app is stable
            // or after 30 seconds (whichever comes first).
            registrationStrategy: 'registerWhenStable:30000'
        }),
        NgbModule,
        GraphQLModule,
        HttpClientModule,
        CdkTableModule,
        ...MatModules,
        FormsModule,
        MatOptionModule,
        ReactiveFormsModule
    ],
    providers: [
        ...Services
    ],
    bootstrap: [AppComponent]
})
export class AppModule { }
